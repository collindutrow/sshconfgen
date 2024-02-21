# sshconfgen
SSH Config Generator. Generate SSH client config based on user defined conditions.

Compatible with Linux, macOS, and Windows.

## Configuration

```shell
mkdir -p ~/.ssh/config.d
```

Create a file in `~/.ssh/config.d/` for your hosts configuration and give it the extension `.sshconf`.

### Example Configuration

If `LocalSSID`, `LocalGateway`, or `LocalPing` conditions match or are reachable, the Local rules will be used.
All conditions are optional, however you will need at least one condition to use the `LOCAL CONFIG`.
You can have as many `.sshconf` files as you want, and they will be processed alphabetically.

* `LocalSSID` comma separated list of SSIDs to match. 
  * (Requires `networksetup` on macOS, `iwgetid` on Linux, `netsh` on Windows)
  <br><br>
* `LocalGateway` comma separated list of `IP|MAC` addresses to match. (Requires `arp`)
<br><br>
* `LocalPing` comma separated list of IP addresses to ping. (Requires `ping`)
  * Note: Ping will cause the biggest delay in runtime completion, so use it sparingly.

`~/.ssh/config.d/00-myconfig.sshconf`:
```
# CONDITIONS BEGIN
LocalSSID foo25ghz, bar5ghz
LocalGateway 192.168.1.1|00:11:22:33:44:55,172.16.1.1|00:55:44:33:22:11
LocalPing 192.168.1.100,172.16.1.100
# CONDITIONS END

# GLOBAL CONFIG BEGIN
Host aws1
	HostName 999.99.999.99
	Port 22
	User ec2-user
# GLOBAL CONFIG END

# LOCAL CONFIG BEGIN
Host homeserver1
    HostName 192.168.1.7
    Port 22
    
Host homeserver2
    HostName 192.168.1.8
    Port 22
# LOCAL CONFIG END

# REMOTE CONFIG BEGIN
Host homeserver1
    HostName mysuperawesomehomeserver1234.dyndns.org
    Port 2221
    
Host homeserver2
    HostName mysuperawesomehomeserver5678.dyndns.org
    Port 2222
# REMOTE CONFIG END
```

Blank template:
```
# CONDITIONS BEGIN
# CONDITIONS END

# GLOBAL CONFIG BEGIN
# GLOBAL CONFIG END

# LOCAL CONFIG BEGIN
# LOCAL CONFIG END

# REMOTE CONFIG BEGIN
# REMOTE CONFIG END
```

## Usage

Run `sshconfgen` to generate a new `~/.ssh/config` file.
```shell
sshconfgen
```
Run with verbose output.
```shell
sshconfgen --verbose
```
Print out detailed help and usage. (Contains more information than this README file.)
```shell
sshconfgen --help
```

## Automation

### macOS

#### LaunchAgent

Save the plist code below as `local.sshconfgen.plist` in `~/Library/LaunchAgents/` 
and run `launchctl bootstrap gui/$UID ~/Library/LaunchAgents/local.sshconfgen.plist` to start the agent.
Note: You will need to adjust the `sshconfgen` path to match your installation.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" \
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
    <dict>
        <key>Label</key>
        <string>local.sshconfgen</string>
        <key>LowPriorityIO</key>
        <true/>
        <key>ProgramArguments</key>
        <array>
            <string>/usr/local/bin/sshconfgen</string>
        </array>
        <key>WatchPaths</key>
        <array>
            <string>/etc/resolv.conf</string>
            <string>/var/run/resolv.conf</string>
            <string>/private/var/run/resolv.conf</string>
            <string>/Library/Preferences/SystemConfiguration/NetworkInterfaces.plist</string>
            <string>/Library/Preferences/SystemConfiguration/com.apple.airport.preferences.plist</string>
        </array>
        <key>RunAtLoad</key>
        <true/>
    </dict>
</plist>
```
