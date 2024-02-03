pub fn print_help() {
    // Get the name of the binary.
    let binding = std::env::current_exe().unwrap();
    let binary = binding.file_name().unwrap().to_str().unwrap();

    println!("{}", format!("Usage: {} [OPTIONS]", binary));
    println!(
"
-h, --help\t\tPrints this help information
-v, --version\t\tPrints version information

This utility generates a new SSH config file by alphabetically parsing
through .sshconf files found in $HOME/.ssh/conf.d/.

The generated file is structured into sections, formatted as follows:
------------------------------------------------
# CONFIG BEGIN
LocalSSID foo, bar5ghz
LocalGateway 192.168.1.1|00:11:22:33:44:55,172.16.1.1|00:55:44:33:22:11
LocalPing 192.168.1.100,172.16.1.100
# CONFIG END
# GLOBAL RULES BEGIN
<global ssh config>
# GLOBAL RULES END
# LOCAL RULES BEGIN
<local ssh config>
# LOCAL RULES END
# REMOTE RULES BEGIN
<remote ssh config>
# REMOTE RULES END
------------------------------------------------
Ensure that the .sshconf files within $HOME/.ssh/conf.d/ are properly formatted to be parsed and
included in the respective sections.

LocalSSID: (Optional) Succeeds if the current SSID matches any of a comma-separated list of SSIDs.

LocalGateway: (Optional) Succeeds if any of a comma-separated key-value pair IP/MAC address matches.
The IP and MAC pairs are separated by a pipe character, and the pairs are separated by commas.

LocalPing: (Optional) Succeeds if any of a comma-separated list of IP addresses are pingable.
Warning: This may cause a delay in the generation of the ssh config file if the IP addresses are
unreachable.

If LocalSSID, LocalGateway, or LocalPing are specified and match or succeed, the contents of the
local rules section will be included in the generated ssh config file, otherwise the remote rules
section will be included.

Global rules are always included in the generated ssh config file.
"
    );
}