# sip_tester
This project is a start for our need to test SIP functionallity through stunnel SSH. It is also me wanting to learn RUST just for the fun of it. Learn by doing, something like that. I try to do small projects like this on my stream, and I might add videos going through them as regular YT movies too. [YouTube](https://www.youtube.com/channel/UCHjoHMIwzAbOYIc5_DWADNQ)

Anyway, for now this program only rotates the hosts in config.yaml on the screen, trying to do the logins you configured.
It will only work through stunnel, and cant go direcly towards your SIP server, that I know. Our customers use koha and access them through a tunnel using stunnel. And thats what Im first of all writing this for, but ill be happy to consider any sugestions and comments.

You have to configure config.yaml in the resource dir and you will have too put it in a dir where it will be found.
The following dirs are searched:
```
./config.yaml
/etc/sip_tester.yaml
/etc/sip_tester/config.yaml
{Home of user}/sip_tester.config.yaml
```
Its easy enough to change in main.rs if you need it someware else, I might add a -c <config> in the future.

The config file is straight forward also. you list servers in the yaml format and under each server you will configure each stunnel port it serves to clients with port, username and password.
```
sipservers:
  192.168.96.1:
    8888:
      port: 8888
      username: "test"
      password: "test"
    8881:
      port: 8881
      username: "test"
      password: "test"
  192.168.96.2:
    6002:
      port: 6002
      username: "test"
      password: "test"
    6003:
      port: 6003
      username: "test"
      password: "test"
alerts:
  slack:
    url: https://slack.com/api/chat.postMessage?token=xoxb-????????&channel=?????????&text={}      
```

Then its just "cargo run" I will try to add executables for ubuntu 20.04 and windows 10 when I figure out how :)

Best of luck, and please drop a line in issues or so if you got any sugestions, etc.

I work in a team responsible for our server environment on imCode Partner AB, and we are always happy to help and meet new customers. 

/Jacob Sandin
