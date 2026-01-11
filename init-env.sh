#!/bin/sh

set -e

echo "Creating development keys and certificates..."

# Get hostname for certificates (default to localhost for dev)

if [ -n "$PLATFORM_HOSTNAME" ]; then
  platform_hostname="$PLATFORM_HOSTNAME"
  echo "Using hostname from environment: $platform_hostname"
else
  read -p "What hostname will be used for platform (default: nerine.localhost)? " -r platform_hostname
  platform_hostname="${platform_hostname:-nerine.localhost}"
fi

# Ask about auto HTTPS for platform
read -p "Attempt auto HTTPS for platform? (y/N): " -r enable_https
case "$enable_https" in
  [Yy]* ) export ENABLE_HTTPS_PLATFORM=yes;;
  * ) ;;
esac

if [ -n "$CHALLS_HOSTNAME" ]; then
  challs_hostname="$CHALLS_HOSTNAME"
  echo "Using hostname from environment: $challs_hostname"
else
  read -p "What hostname will be used for challs (default: challs.localhost)? " -r challs_hostname
  challs_hostname="${challs_hostname:-challs.localhost}"
fi

if [ -n "$CHALLS_IP" ]; then
  challs_ip="$CHALLS_IP"
  echo "Using IP address from environment: $challs_ip"
else
  read -p "What IP address will be used for challs externally (default: 172.17.0.1)? " -r challs_ip
  challs_ip="${challs_ip:-172.17.0.1}"
fi

if [ -n "$CA_CN" ]; then
  ca_cn="$CA_CN"
  echo "Using CA CN from environment: $ca_cn"
else
  read -p "What CN will be used for CA (default: dev-ca)? " -r ca_cn
  ca_cn="${ca_cn:-dev-ca}"
fi

# Check if keys directory exists and ask about overwriting
generate_keys=true
if [ -d "keys" ]; then
  read -p "Keys directory already exists. Overwrite existing keys? (y/N): " -r overwrite_keys
  case "$overwrite_keys" in
    [Yy]* ) generate_keys=true;;
    * ) generate_keys=false;;
  esac
fi

# Create keys directory structure
mkdir -p keys/docker
mkdir -p keys/caddy

if [ "$generate_keys" = true ]; then
  ###################
  ### Docker certs ###
  ###################

  echo "Generating Docker certificates..."
  cd keys/docker

  # Create CA
  openssl genrsa -out ca-key.pem 4096
  openssl req -new -x509 -days 365 -key ca-key.pem -sha256 -out ca.pem <<EOF 2> /dev/null
.
.
.
.
.
$ca_cn
.
EOF

  # Generate server key & cert signing request
  openssl genrsa -out server-key.pem 4096
  openssl req -subj "/CN=docker" -sha256 -new -key server-key.pem -out server.csr
  cat >extfile.cnf <<EOF
subjectAltName = DNS:$challs_hostname,IP:$challs_ip,IP:127.0.0.1
extendedKeyUsage = serverAuth
EOF

  # Sign server cert
  openssl x509 -req -days 365 -sha256 -in server.csr -CA ca.pem -CAkey ca-key.pem -CAcreateserial -out server-cert.pem -extfile extfile.cnf

  # Create client key & cert signing request
  openssl genrsa -out key.pem 4096
  openssl req -subj '/CN=client' -new -key key.pem -out client.csr
  cat >extfile-client.cnf <<EOF
extendedKeyUsage = clientAuth
EOF

  # Sign client cert
  openssl x509 -req -days 365 -sha256 -in client.csr -CA ca.pem -CAkey ca-key.pem -CAcreateserial -out cert.pem -extfile extfile-client.cnf

  echo "Docker certificates created in keys/docker/"

  ##################
  ### Caddy certs ###
  ##################

  echo "Generating Caddy certificates..."
  cd ../caddy

  # Create CA
  openssl genrsa -out ca-key.pem 4096
  openssl req -new -x509 -days 365 -key ca-key.pem -sha256 -out ca.pem <<EOF 2> /dev/null
.
.
.
.
.
$ca_cn
.
EOF

  # Generate server key & cert signing request
  openssl genrsa -out server-key.pem 4096
  openssl req -subj "/CN=caddy" -sha256 -new -key server-key.pem -out server.csr
  cat >extfile.cnf <<EOF
subjectAltName = DNS:$challs_hostname,IP:$challs_ip,IP:127.0.0.1
extendedKeyUsage = serverAuth
EOF

  # Sign server cert
  openssl x509 -req -days 365 -sha256 -in server.csr -CA ca.pem -CAkey ca-key.pem -CAcreateserial -out server-cert.pem -extfile extfile.cnf

  # Create client key & cert signing request
  openssl genrsa -out key.pem 4096
  openssl req -subj '/CN=client' -new -key key.pem -out client.csr
  cat >extfile-client.cnf <<EOF
extendedKeyUsage = clientAuth
EOF

  # Sign client cert
  openssl x509 -req -days 365 -sha256 -in client.csr -CA ca.pem -CAkey ca-key.pem -CAcreateserial -out cert.pem -extfile extfile-client.cnf

  echo "Caddy certificates created in keys/caddy/"

  cd ../..
else
  echo "Skipping key generation, using existing keys..."
fi

# Generate keychain.json
echo ""
echo "Generating keychain.json..."

read_pem_json() {
  <"$1" sed 's/\\/\\\\/g; s/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g'
}

cd keys
cat <<EOF > ./keychain.json
[{
  "id": "default",
  "caddy": {
    "endpoint": "https://$challs_ip:995",
    "base": "$challs_hostname",
    "cacert": "$(read_pem_json caddy/ca.pem)",
    "cert": "$(read_pem_json caddy/cert.pem)",
    "key": "$(read_pem_json caddy/key.pem)"
  },
  "docker": {
    "docker": {
      "type": "local"
    },
    "docker_external_config_removethistouse": {
      "type": "ssl",
      "address": "$challs_ip:996",
      "ca": "$(read_pem_json docker/ca.pem)",
      "cert": "$(read_pem_json docker/cert.pem)",
      "key": "$(read_pem_json docker/key.pem)"
    },
    "docker_credentials_removethistouse": {
      "username": "<docker-registry-username>",
      "password": "<docker-registry-password>",
      "serveraddress": "<docker-registry-address>"
    },
    "docker_credentials": null,
    "image_prefix": "",
    "repo": "ghcr.io/ctf-gg/nerine"
  }
}]
EOF
cd ..

echo ""
if [ "$generate_keys" = true ]; then
  echo "All certificates generated successfully!"
  echo "Keys are located in ./keys/"
  echo "  - Docker certs: ./keys/docker/"
  echo "  - Caddy certs: ./keys/caddy/"
  echo ""
fi
echo "keychain.json has been created in the current directory."
echo "Edit it to add docker registry credentials if needed otherwise the local docker daemon will be used."

echo "Generating caddy config..."
mkdir -p docker
EXTERNAL_IP=172.17.0.1 ENABLE_HTTPS_PLATFORM=no ADD_PLATFORM_ROUTES=1 python3 scripts/generate_caddy_config.py nerine.localhost challs.localhost ./keys > docker/caddy_dev_localhost.json
EXTERNAL_IP=$challs_ip ADD_PLATFORM_ROUTES=1 python3 scripts/generate_caddy_config.py $platform_hostname $challs_hostname ./keys > docker/caddy_dev.json
echo "Generating caddy config for chall machine only..."
EXTERNAL_IP=$challs_ip python3 scripts/generate_caddy_config.py $platform_hostname $challs_hostname ./keys > docker/caddy.json
# extra config for daisychain use case, for hacky situations, do not rely on this.
EXTERNAL_IP=$challs_ip BIND_IP=172.17.0.1 HTTP_PORT=8080 TRUST_PROXY=yes python3 scripts/generate_caddy_config.py $platform_hostname $challs_hostname ./keys > docker/caddy_daisychain.json
echo "Generating simple Caddyfile config..."
python3 scripts/generate_simple_caddyfile_config.py $platform_hostname > docker/Caddyfile.platform
python3 scripts/generate_env_with_secrets.py > .env
echo "Done."
echo "Go configure .env and make sure keys/keychain.json does what you want."