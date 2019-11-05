extern crate sse_client;
extern crate json;
extern crate mysql;
extern crate chrono;
extern crate config;
#[macro_use]
extern crate log;
extern crate simplelog;

use sse_client::EventSource;
use mysql as my;
use chrono::Local;
use simplelog::*;


fn main() {
    // Setup logger
    SimpleLogger::new(LevelFilter::Info, Config::default());
    info!("Program started");

    // Read settings
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("/etc/eventsource/eventsource.toml")).unwrap();

    // REST API and Server-sent events
    // Connect to the event source from OpenHAB
    // https://www.openhab.org/docs/configuration/restdocs.html
    let url_str = format!("{}/rest/events?topics=smarthome/items/*/statechanged", settings.get_str("eventsource_url").unwrap());
    debug!("{:?}", url_str);
    let event_source = EventSource::new(&url_str).unwrap();

    for event in event_source.receiver().iter() {
        debug!("New Message: {}", event.data);
        
        // Clean-up the event message before letting the json parser consume it
        let cleaned_up_str = event.data
            .replace("\\", "")
            .replace("\"{", "{")
            .replace("}\"", "}")
            .replace("\"[", "[")
            .replace("]\"", "]");

        let event_json = json::parse(&cleaned_up_str);
        let event_json = match event_json {
            Ok(ev_json) => ev_json,
            Err(error) => {
                error!("Error: {:?} when parsing: {}", error, cleaned_up_str);
                continue;
            },
        };

        debug!("{:#}", event_json);

        // We are only interested in stae changing events
        if event_json["type"] == "ItemStateChangedEvent"  {

            // Setup the database conenction
            let mysql_str = format!("mysql://{}:{}@localhost:3306/sensor", 
            settings.get_str("mysql_user").unwrap(),
            settings.get_str("mysql_passwd").unwrap() );

            match my::Pool::new(mysql_str) {
                Ok(pool) => {
                    let current_time_stamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

                    let mut data = event_json["payload"]["value"].to_string();                    
                    let topic = event_json["topic"].as_str();
                    match topic {
                        Some(t) => {
                            // The Tri-sensor sometimes sends negative values when very bright outside
                            if t.contains("Luminance") {
                                data = data.replace("-", "");
                            }
                        },
                        None =>  error!("Error: Could not find a topic in: {}",  event.data),
                    }
                
                    // Put the event into the database
                    match pool.prep_exec("INSERT INTO event (name, 
                                                             timeStamp, 
                                                             what, 
                                                             data)
                                          VALUES (?, ?, ?, ?)", 
                        (
                            event_json["topic"].as_str(), 
                            current_time_stamp, 
                            event_json["payload"]["type"].as_str(), 
                            data
                        ))
                    {
                        Ok(result) => {
                            debug!("Data added id: {}", result.last_insert_id());
                        }
                        Err(err) => {
                            error!("Error: {:?}", err);
                        },
                    }
                },
                Err(err) => {
                    error!("Cannot connect to database: {:?}", err);
                },
            }
        }
    }
}
