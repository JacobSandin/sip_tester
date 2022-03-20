extern crate native_tls;
extern crate dirs;

use native_tls::TlsConnector;//, TlsStream, HandshakeError};
use std::io::{Read, Write, BufReader, BufRead};
use std::net::TcpStream;
use std::fs::File;
use std::path::Path;
use yaml_rust::YamlLoader;
use std::{thread, time};
use url::Url;
use std::process::Command;

fn load_file(file: &str) {
    println!("File: {}",file);
    let mut my_file = File::open(&file).expect("Unable to open file");
    let mut contents = String::new();
    my_file.read_to_string(&mut contents).expect("Unable to read file");

    let yamldocs = YamlLoader::load_from_str(&contents)
        .expect(&format!("Expected parsable configuration in file {}",file));
    let yamldoc = &yamldocs[0];

    let servers = yamldoc["sipservers"].as_hash().expect("One or more SIP servers in config!");
    
    let alerts = &yamldoc["alerts"]["slack"]["url"].as_str().unwrap_or("");

    loop {
        for (server,server_values) in servers.iter() {
            for (_port, config) in server_values.as_hash().expect("Invalid sip_server config").iter() {
                println!("{}:{} -u {} -p {} stunnel:{} message:{}",
                         server.as_str().unwrap_or("127.0.0.1"),
                         config["port"].as_i64().unwrap_or(8881),
                         config["username"].as_str().unwrap_or("test"),
                         config["password"].as_str().unwrap_or("gurka"),
                         config["stunnel"].as_str().unwrap_or("true") == "true",
                         config["message"].as_str().unwrap_or("Message?")
                         );
                if config["stunnel"].as_str().unwrap_or("true") == "true" { 
                    if !try_login_to_sip_tls(server.as_str().unwrap_or("127.0.0.1"),
                                         config["port"].as_i64().unwrap_or(8881),
                                         config["username"].as_str().unwrap_or("test"),
                                         config["password"].as_str().unwrap_or("gurka")) {
                        println!("=========== ERROR! ============");
    
                        //TODO send config
                        if !alerts.is_empty() {
                            handle_error(&format!("SIP Falied to login: {}:{} ({})", 
                                server.as_str().unwrap_or("127.0.0.1"),
                                config["port"].as_i64().unwrap_or(8881),config["message"].as_str().unwrap_or("dont have message")),
                                &alerts);
                        }
                    }
                } else {
                    if !try_login_to_sip(server.as_str().unwrap_or("127.0.0.1"),
                                         config["port"].as_i64().unwrap_or(8881),
                                         config["username"].as_str().unwrap_or("test"),
                                         config["password"].as_str().unwrap_or("gurka")) {
                        println!("=========== ERROR! ============");
    
                        //TODO send config
                        if !alerts.is_empty() {
                            handle_error(&format!("SIP Falied to login: {}:{} ({})", 
                                server.as_str().unwrap_or("127.0.0.1"),
                                config["port"].as_i64().unwrap_or(8881),config["message"].as_str().unwrap_or("dont have message")),
                                &alerts);
                        }
                    }
                }
            }
        }
        let sleep_for = time::Duration::from_millis(60000);
        thread::sleep(sleep_for);
    }
}

fn try_login_to_sip_tls(server: &str, port: i64,username: &str, password: &str) -> bool {
    // Make a builder to accept invalid certs of the stunnel servers wich I have not bothered geting root from.
    let builder = TlsConnector::builder().danger_accept_invalid_certs(true).build().expect("Expected to be able to accept all certs.");
    //Start a streamobject with the server and port

    //TODO Error handling
    let stream = match TcpStream::connect(format!("{}:{}",server,port)){
        Ok(stream) => stream,
        Err(error) => {
            println!("Error: {:?}",error);
            return false
        },
    };


        let mut stream = match builder.connect(server, stream) {
            Ok(stream) => stream,
            Err(error) => {
                println!("Error: {:?}", error);
                return false
            },
        };
    
    let base_string = format!("9300CN{}|CO{}|AY0AZ", username, password);
    let crc_string = &do_crc(String::from(base_string.clone()));
    let login_string = format!("{}{}",base_string, crc_string);
    println!("LoginString TLS: {}",login_string);
    match stream.write_all(format!("{}\r\n\r\n",login_string).as_bytes()) {
        Err(error) => {
            println!("Error: {:?}", error);
            return false
        },
        _ => (),
    }

    let mut res = String::new();

    //TODO error handling

    let mut buff = BufReader::new(stream);
    match buff.read_line(&mut res) {
        Err(error) => {
            println!("Error: {:?}", error);
            false
        },
        _ => {
            println!("-- {:?} --\r\n\r\n", res.trim_end());
            if res.trim_end() == "940AY0AZFDFE" || res.trim_end() == "96AZFEF6" {
                return true
            }
            false
        },
    };
    false
}

/**
 * I got this part from https://github.com/tzeumer/SIP2-Client-for-Python/blob/master/Sip2/sip2.py
 * TODO Get it to work, it might not be configured right in KOHA because
 * even with this checksum it do return 940 not 941 as is.
 * https://docs.evergreen-ils.org/2.6/_sip_communication.html
 */
fn do_crc(msg: String) -> String {
    //let msg = String::from( "09N20160419    12200820160419    122008APReading Room 1|AO830|AB830$28170815|AC|AY2AZ"); //crc should be EB80
    let mut str_sum: i32 = 0;
    for c in msg.chars() {
        str_sum += c as i32;
    }
    //println!("{:X}", -str_sum & 0xFFFF);
    return format!("{:X}", -str_sum & 0xFFFF);
}


fn try_login_to_sip(server: &str, port: i64,username: &str, password: &str) -> bool {
    //Start a streamobject with the server and port

    //TODO Error handling
    let mut stream = match TcpStream::connect(format!("{}:{}",server,port)){
        Ok(stream) => stream,
        Err(error) => {
            println!("Error: {:?}",error);
            return false
        },
    };

    let base_string = format!("9300CN{}|CO{}|AY0AZ", username, password);
    let crc_string = &do_crc(String::from(base_string.clone()));
    let login_string = format!("{}{}",base_string, crc_string);
    println!("LoginString: {}",login_string);
    match stream.write_all(format!("{}\r\n\r\n",login_string).as_bytes()) {
        Err(error) => {
            println!("Error: {:?}", error);
            return false
        },
        _ => (),
    }

    let mut res = String::new();

    //TODO error handling

    let mut buff = BufReader::new(stream);
    match buff.read_line(&mut res) {
        Err(error) => {
            println!("Error: {:?}", error);
            false
        },
        _ => {
            println!("-- {:?} --\r\n\r\n", res.trim_end());
            if res.trim_end() == "940AY0AZFDFE" || res.trim_end() == "96AZFEF6" {
                return true
            }
            false
        },
    };
    false
}

fn alert_slack(full_url: &str) {

    let url = match Url::parse(full_url) {
        Ok(url) => url,
        Err(error) => {
            println!("Slack url parse error: {}", error);
            return ()
        },
    };
    let server = url.host_str().unwrap_or("localhost");
    let port = url.port_or_known_default().unwrap_or(80);
    let query = format!("{}?{}",url.path(),url.query().unwrap_or(""));

    println!("url: {} {} {}",server, port, query );


    let builder = TlsConnector::builder().danger_accept_invalid_certs(true).build().unwrap();
    let stream = match TcpStream::connect(format!("{}:{}",server,port)){
        Ok(stream) => stream,
        Err(error) => {
            println!("Error: {:?}",error);
            return
        },
    };
    let mut stream = match builder.connect(server, stream) {
        Ok(stream) => stream,
        Err(error) => {
            println!("Error: {:?}", error);
            return
        },
    };

    match stream.write_all(format!("GET {} HTTP/1.2\r\nHost: {}\r\n\r\n",query,server).as_bytes()) {
        Err(error) => {
            println!("Error: {:?}", error);
            return
        },
        _ => (),
    }

    let mut res = String::new();

    //TODO error handling

    let mut buff = BufReader::new(stream);
    match buff.read_line(&mut res) {
        Err(error) => {
            println!("Error: {:?}", error);
            return
        },
        _ => {
            println!("-- {:?} --", res.trim_end());
            return
        },
    };

}

// TODO: 
fn handle_error(message: &str, slackurl: &str) {
    if !slackurl.contains("http") {
       if let Err(e) = Command::new(&slackurl)
            .arg(&message)
            .output() 
       {
                println!("Error: {:?}", e);
       }
    } else {
        let slack_url = str::replace(&slackurl,"{}",&message);
        alert_slack(&slack_url);
    }
}

fn main() {

    let home_dir = String::from(dirs::home_dir().unwrap().to_str().unwrap());

    if Path::new("./config.yaml").exists() {
        println!("Using config: {}","./config.yaml");
        load_file("config.yaml");
        return
    } else if Path::new("/etc/sip_tester.yaml").exists() {
        println!("Using config: {}","/etc/sip_tester.yaml");
        load_file("/etc/sip_tester.yaml");
        return
    } else if Path::new("/etc/sip_tester/config.yaml").exists() {
        println!("Using config: {}","/etc/sip_tester/config.yaml");
        load_file("/etc/sip_tester/config.yaml");
        return
    }else if Path::new(&format!("{}{}",&home_dir, "/.sip_tester.config.yaml")).exists() {
        println!("Using config: {}",&format!("{}{}",&home_dir, "/.sip_tester.config.yaml"));
        load_file(&format!("{}{}",&home_dir, "/.sip_tester.config.yaml"));
        return
    }

    println!("No config file found, searched (./config.yaml) (/etc/sip_tester.yaml) and (/etc/sip_tester/config.yaml) ({})",&format!("{}{}",&home_dir, "/.sip_test.config.yaml"));
}
