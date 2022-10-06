#!/bin/sh

# Assumes nats, geyser-plugin and indexer are installed and built
# Also the following packages build-essentials pkg-config libssl-dev

#Run as root

echo SOLANA_NETWORK="devnet" >> /etc/environment #Choose from 'testnet', 'devnet', or 'mainnet-beta'
export ELASTIC_ENROLLMENT_TOKEN="TGJ2Y3JvTUIxWGROMXJUd2I1c0M6Q1FXYlltRjhUOEs2MWwxeFludzlUZw=="

#Ensure your PC is updated and upgraded to the latest version
apt update
apt upgrade -y
#apt dist-upgradevault read -field=public_key ssh-client-signer/config/ca > /etc/ssh/trusted-user-ca-keys.pem


apt install vim screen htop curl ufw jq -y

#Enable Firewall
ufw allow ssh
ufw allow 53
ufw allow 8000:8020/udp
ufw allow 8000:8020/tcp
# ufw allow 8899
ufw allow 8900
ufw enable

#Add the Solana sol user, then make the solana user a sudoer. Make sure to save the password. 
adduser sol
usermod -aG sudo sol

#Deny sol user SSH access
echo '
DenyUsers sol' >> /etc/ssh/sshd_config

#Create the ledger & swap filesystems. The nvme1n1 drive is the smaller of the two,
#with nvme1n1 and nvme0n1 having about 1TB and 2TB of storage respectively.
mkswap /dev/nvme1n1
mkfs -t ext4 /dev/nvme0n1

#Create the Ledger Mount directory /mnt/solana-ledger  and the Account directory /mnt/solana-accounts
mkdir /mnt/solana-ledger
mkdir /mnt/solana-accounts

#Configure the system to mount the filesystems
swapoff /swapfile
sed -i '/swapfile/s/^/#/' /etc/fstab
echo "
/dev/nvme1n1 none swap sw 0 0
/dev/nvme0n1 /mnt/solana-ledger ext4 defaults 0 0
tmpfs /mnt/solana-accounts tmpfs rw,size=200G,user=sol 0 0
" >> /etc/fstab
#If your system memory is less than 512GB, make the size of the tmpfs 200G instead of 300G on that last line.

#Mount the newly added fstab items and enable the new swap
mount -a
swapon --all --verbose

#Check that the swap, ledger, and accounts filesystems are mounted
mount
swapon --show

#Set the scaling governor on the highest performance settings
echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

#Configure use of swap only when 15% or less of memory is free
echo 'vm.swappiness=10' | sudo tee --append /etc/sysctl.conf > /dev/null

#Create the log directory and pass the log and ledger directories' ownership to the  sol  user
mkdir /var/log/sol
chown -R sol:sol /mnt/solana-ledger
chown -R sol:sol /var/log/sol

cp /home/caleb/bokoup-program-library/validator-setup/solana_validator_setup.sh /home/sol/
cp /home/caleb/bokoup-program-library/target/release/bpl-indexer /home/sol/
cp /home/caleb/geyser-plugin-nats/target/release/libgeyser_plugin_nats.so /home/sol/

echo '{
    "libpath": "/home/sol/libgeyser_plugin_nats.so",
    "addresses": [
        "FtccGbN7AXvtqWP5Uf6pZ9xMdAyA7DXBmRQtmvjGDX7x",
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
        "hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk"
    ]
}' > /home/sol/config.json

chown -R sol:sol /home/sol/
chmod -R 711 /home/sol/solana_validator_setup.sh
su - sol -s solana_validator_setup.sh

#Create the nats service
echo '[Unit]
[Unit]
Description=nats-server service
ConditionPathExists=/usr/local/bin/nats-server
After=network.target

[Service]
Type=simple
Restart=always
RestartSec=1
User=sol
ExecStart=/usr/local/bin/nats-server

[Install]
WantedBy=multi-user.target' > /etc/systemd/system/nats.service

#Create the indexer service
echo '[Unit]
Description=bpl-indexer service
ConditionPathExists=/usr/local/bin/bpl-indexer
After=nats.service

[Service]
Type=simple
Restart=always
RestartSec=1
User=sol

Environment=PG_PASSWORD_LOCALNET=

Restart=always

ExecStart=/home/caleb/bokoup-program-library/target/release/bpl-indexer

StandardOutput=syslog
StandardError=inherit
SyslogIdentifier=SyslogIdentifier=bpl-indexer

[Install]
WantedBy=multi-user.target ' > /etc/systemd/system/bpl-indexer.service

#Create the solana-sys-tuner.service
echo '[Unit]
Description=Solana System Tuner
After=network.target

[Service]
Type=simple
Restart=on-failure
RestartSec=1
LogRateLimitIntervalSec=0
ExecStart=/home/sol/.local/share/solana/install/active_release/bin/solana-sys-tuner --user sol

[Install]
WantedBy=multi-user.target' > /etc/systemd/system/solana-sys-tuner.service

#Create the sol service
echo '[Unit]
Description=Solana Validator
After=network.target
After=nats.service
Wants=solana-sys-tuner.service
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=1
User=sol
LimitNOFILE=1000000
LogRateLimitIntervalSec=0
Environment="PATH=/bin:/usr/bin:/home/sol/.local/share/solana/install/active_release/bin"
ExecStart=/home/sol/bin/validator-devnet.sh
ExecStop=/home/sol/.local/share/solana/install/active_release/bin/solana-validator --ledger /mnt/solana-ledger/ exit -f

[Install]
WantedBy=multi-user.target' > /etc/systemd/system/sol.service

#Install Elastic Agent
curl -L -O https://artifacts.elastic.co/downloads/beats/elastic-agent/elastic-agent-8.4.3-linux-x86_64.tar.gz
tar xzvf elastic-agent-8.4.3-linux-x86_64.tar.gz
cd elastic-agent-8.4.3-linux-x86_64
sudo ./elastic-agent install --url=https://a5231615528443d5acdeb602d50f7254.fleet.us-central1.gcp.cloud.es.io:443 --enrollment-token=$ELASTIC_ENROLLMENT_TOKEN

systemctl daemon-reload
systemctl enable --now nats
systemctl enable --now bpl-indexer
systemctl enable --now solana-sys-tuner
systemctl enable --now sol

#Enable log rotation for validator.log
cat > /etc/logrotate.d/sol <<EOF
/var/log/sol/validator.log {
rotate 7
daily
missingok
postrotate
systemctl kill -s USR1 sol.service
endscript
}
EOF

systemctl restart logrotate.service
