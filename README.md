# SSPAGBOL
A command-line information tool for your services written in Rust with support for Linux & Windows. Easily configured via a YAML file. SSPAGBOL allows you to easily check which services are currently running from one command.

![alt text](https://raw.githubusercontent.com/afit21/sspagbol/refs/heads/master/images/screenshot.png) 

SPAGBOL is currently in a very early state, so watch this space!

# Supported Protocols
SSPAGBOL can currently evaluate the status of the following; ping, HTTP/HTTPS, SSH, and DNS.
## Example YAML
### Hostmachine (Ping)
```
- name: Machine
  desc: Example Machine
  cilist:
    - ciname: Hostmachine
      citype: Hostmachine
      cidata:
        - "192.168.0.10"
```
### Webserver (HTTP/HTTPS)
```
- name: Webserver
  desc: Example Webserver
  cilist:
    - ciname: HTTPS Web Server
      citype: Webserver
      cidata:
        - "https://google.com/"
        - "443"
```
### SSH
```
- name: SSH
  desc: Example SSH Server
  cilist:
    - ciname: SSH Connection
      citype: SSHServer
      cidata:
        - "192.168.0.11"
        - "22"
```
### DNS
```
- name: DNS
  desc: Example DNS Server
  cilist:
    - ciname: Example DNS Server
      citype: DNSServer
      cidata:
        - "8.8.8.8"
```
