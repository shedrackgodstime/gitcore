]
└──╼ $echo $0
echo $SHELL
ps -p $$
/bin/bash
/bin/bash
PID TTY          TIME CMD
2207 pts/1    00:00:00 bash
┌─[kristency@parrot]─[~]
└──╼ $nano ~/.bashrc
┌─[kristency@parrot]─[~]
└──╼ $source ~/.bashrc
┌─[kristency@parrot]─[~]
└──╼ $gity list
No accounts configured. Run 'gity add <name> <platform>'
┌─[kristency@parrot]─[~]
└──╼ $gity add
Enter account name: shedrackgodstime
Enter platform (github/gitlab/codeberg/bitbucket): github
Enter your git username (for commits): shedrackgodstime
Enter your email (for SSH key + commits): shedrackgodstime@gmail.com
Enter passphrase for SSH key (leave empty for no protection): @Kristen12
Generating SSH key...
✓ SSH key generated
✓ Key is protected with passphrase
✓ SSH config updated

┌─────────────────────────────────────────────────┐
│           Account Added Successfully!           │
└─────────────────────────────────────────────────┘

Name:     shedrackgodstime
Platform: Github
Use:      git clone git@github-shedrackgodstime:user/repo.git

───────────────────────────────────────────────────

1. ADD SSH KEY TO YOUR PLATFORM

ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIC6CU8s3sjbvqWA9PqfAKDwd5zjWCgZDQF0mDKfDhcKt shedrackgodstime@gmail.com


Open: https://github.com/settings/keys

───────────────────────────────────────────────────

2. TEST CONNECTION

Run: gity test github-shedrackgodstime

───────────────────────────────────────────────────

3. USAGE

Clone:  git clone git@github-shedrackgodstime:username/repo.git
Remote: gity remote add

┌─[kristency@parrot]─[~]
└──╼ $gity test github-shedrackgodstime

Testing connection to github.com...
⚠ New host key - will be added to known_hosts
✗ Connection failed
git@github.com: Permission denied (publickey).
┌─[kristency@parrot]─[~]
└──╼ $gity test github-shedrackgodstime

Testing connection to github.com...
⚠ New host key - will be added to known_hosts
✗ Connection failed
git@github.com: Permission denied (publickey).
┌─[kristency@parrot]─[~]
└──╼ $sudo gity test github-shedrackgodstime
[sudo] password for kristency:
sudo: gity: command not found
┌─[✗]─[kristency@parrot]─[~]
└──╼ $

installation script is shit
