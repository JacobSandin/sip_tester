extern crate native_tls;
extern crate dirs;

use native_tls::{TlsConnector};//, TlsStream, HandshakeError};
use std::io::{Read, Write, BufReader, BufRead};
use std::net::{TcpStream};
use std::fs::{File};
use std::path::Path;
use yaml_rust::{YamlLoader};
// use rodio::{Sink, Source};
use std::{thread, time};
use url::{Url};



fn load_file(file: &str) {
    println!("File: {}",file);
    let mut file = File::open(file).expect("Unable to open file");
    let mut contents = String::new();

    file.read_to_string(&mut contents).expect("Unable to read file");

    let yamldocs = YamlLoader::load_from_str(&contents).unwrap();
    let yamldoc = &yamldocs[0];

    let servers = yamldoc["sipservers"].as_hash().unwrap();


    let alerts = &yamldoc["alerts"]["slack"]["url"].as_str().unwrap_or("");
    // println!("Alert: {}",&alerts);
    //  handle_error("WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWHAT"
    //               ,&alerts);

    loop {
        for (server,server_values) in servers.iter() {
            for (_port, config) in server_values.as_hash().unwrap().iter() {
                //println!("{:?}",config);
                println!("{}:{} -u {} -p {}",
                         server.as_str().unwrap_or("127.0.0.1"),
                         config["port"].as_i64().unwrap_or(8881),
                         config["username"].as_str().unwrap_or("test"),
                         config["password"].as_str().unwrap_or("gurka"));
                if !try_login_to_sip(server.as_str().unwrap_or("127.0.0.1"),
                                     config["port"].as_i64().unwrap_or(8881),
                                     config["username"].as_str().unwrap_or("test"),
                                     config["password"].as_str().unwrap_or("gurka")) {
                    println!("=========== ERROR! ============");

                    //TODO send config
                    handle_error(&format!("Falied to login: {}:{}", server.as_str().unwrap_or("127.0.0.1"),config["port"].as_i64().unwrap_or(8881))
                                 ,&alerts);
                }
            }
        }
        let sleep_for = time::Duration::from_millis(10000);
        thread::sleep(sleep_for);
        //return
    }
}


fn try_login_to_sip(server: &str, port: i64,username: &str, password: &str) -> bool {
    // Make a builder to accept invalid certs of the stunnel servers wich I have not bothered geting root from.
    let builder = TlsConnector::builder().danger_accept_invalid_certs(true).build().unwrap();
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

    match stream.write_all(format!("9300CN{}|CO{}|CP|AY0AZF535\r\n\r\n", username, password).as_bytes()) {
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
            println!("-- {:?} --", res.trim_end());
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
    let slack_url = str::replace(&slackurl,"{}",&message);
    alert_slack(&slack_url);
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
    }else if Path::new(&format!("{}{}",&home_dir, "/sip_tester.config.yaml")).exists() {
        println!("Using config: {}",&format!("{}{}",&home_dir, "/sip_tester.config.yaml"));
        load_file(&format!("{}{}",&home_dir, "/sip_tester.config.yaml"));
        return
    }

    println!("No config file found, searched (./config.yaml) (/etc/sip_tester.yaml) and (/etc/sip_tester/config.yaml) ({})",&format!("{}{}",&home_dir, "/etc/sip_test.config.yaml"));
}
