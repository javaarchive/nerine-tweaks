#!/usr/bin/env python3
# not as good as voxal at writing bash scripts

import sys
import os
import urllib3

def env_boolean(name: str) -> bool:
    if name in os.environ:
        return os.environ[name].lower() in ("1", "true", "yes")
    return False

def read_file(filename: str) -> str:
    with open(filename, "r") as f:
        return f.read()

# ported from bash script
def read_pem_content(filepath) -> str:
    with open(filepath, 'r') as f:
        lines = f.readlines()
    
    return ''.join(lines[1:-1])

def bail(message: str):
    sys.stderr.write(message + "\n")
    sys.stderr.flush()
    sys.exit(1)

if len(sys.argv) != 4:
    bail("Usage: python3 generate_caddy_config.py <platform-domain> <challs-domain> <keys-dir>")

if not os.path.isdir(sys.argv[3]):
    bail(f"Keys directory {sys.argv[3]} does not exist")

main_filename, platform_domain, challs_domain, keys_dir = sys.argv

ENABLE_HTTPS = env_boolean("ENABLE_HTTPS_PLATFORM")
ENABLE_CF_DNS_CHALLENGES = env_boolean("ENABLE_CF_DNS_CHALLENGES")
ADD_PLATFORM_ROUTES = env_boolean("ADD_PLATFORM_ROUTES")
TRUST_PROXY = env_boolean("TRUST_PROXY")
EXTERNAL_IP = os.environ.get("EXTERNAL_IP")

HTTP_PORT = os.environ.get("HTTP_PORT", 80)
HTTPS_PORT = os.environ.get("HTTPS_PORT", 443)
BIND_HOST = os.environ.get("BIND_HOST", "") # blank will make all interfaces available

trusted_ranges = [
    "192.168.0.0/16",
    "172.16.0.0/12",
    "10.0.0.0/8",
    "127.0.0.1/8",
    "fd00::/8",
    "::1"
]

caddy_admin_public_key = read_pem_content(f"{keys_dir}/caddy/cert.pem")
caddy_admin_identifiers = ["127.0.0.1", "172.17.0.1"]

if EXTERNAL_IP and not EXTERNAL_IP in caddy_admin_identifiers:
    caddy_admin_identifiers.append(EXTERNAL_IP)

config = {
  "admin": {
    "listen": "localhost:990",
    "remote": {
      "listen": "0.0.0.0:995",
      "access_control": [{
        "public_keys": [caddy_admin_public_key]
      }]
    },
    "identity": {
      "identifiers": caddy_admin_identifiers,
      "issuers": [{
        "module": "internal",
        "ca": "local-admin",
        "sign_with_root": True,
      }]
    }
  },
  "apps": {
    "http": {
      "servers": {
        "srv0": {
          "@id": "default-server",
          "automatic_https": {
            "disable": True
          },
          # note is unsupported to have custom ports with https on
          # in certain conditions where caddy cannot figure out
          # which port should be http and which port should be https.
          "listen": [
              f"{BIND_HOST}:{HTTPS_PORT}",
              f"{BIND_HOST}:{HTTP_PORT}"
          ] if ENABLE_HTTPS else [
            f"{BIND_HOST}:{HTTP_PORT}",
          ],
          "routes": [
            {
              "match": [{
                "host": [f"*.{challs_domain}"]
              }],
              "handle": [
                {
                  "handler": "dynamic_router"
                },
                {
                  "handler": "reverse_proxy",
                  "upstreams": [{
                    "dial": "{http.vars.dynamic.upstream}"
                  }]
                }
              ]
            },
            {
              "match": [{
                  "host": [platform_domain]
              }],
              "handle": [{
                "handler": "subroute",
                "routes": [
                  {
                    "handle": [{
                      "handler": "reverse_proxy",
                      "upstreams": [{
                        "dial": "127.0.0.1:3333"
                      }]
                    }],
                    "match": [{
                      "path": ["/api/*"]
                    }]
                  },
                  {
                    "handle": [{
                      "handler": "reverse_proxy",
                      "upstreams": [{
                        "dial": "127.0.0.1:3334"
                      }]
                    }],
                    "match": [{
                      "path": ["/*"]
                    }]
                  }
                ]
              }],
              "terminal": True
            },
            {
              "match": [
                {
                  "host": [
                    f"*.{challs_domain}"
                  ]
                }
              ],
              "terminal": True
            }
          ]
        }
      }
    },
    "pki": {
      "certificate_authorities": {
        "local-admin": {
          "name": "local-admin",
          "install_trust": False,
          "root": {
            "certificate": "/var/lib/caddy/ca.pem",
            "private_key": "/var/lib/caddy/ca-key.pem"
          }
        }
      }
    }
  }
}

if not ADD_PLATFORM_ROUTES:
    # TODO: make this not a hardcoded index
    del config["apps"]["http"]["servers"]["srv0"]["routes"][1]

if TRUST_PROXY:
    config["apps"]["http"]["servers"]["srv0"]["trusted_proxies"] = {
        "ranges": trusted_ranges
    }

if ENABLE_CF_DNS_CHALLENGES:
    # add dns
    config["apps"]["tls"] = {
        "automation": {
            "policies": [
                {
                    "subjects": [
                        f"{platform_domain}",
                        f"*.{challs_domain}"
                    ],
                    "issuers": [
                        {
                            "challenges": {
                                "dns": {
                                    "provider": {
                                        "api_token": "{env.CF_API_TOKEN}", # must provide via env
                                        "name": "cloudflare"
                                    },
                                    "resolvers": [
                                        "1.1.1.1"
                                    ]
                                }
                            },
                            "module": "acme",
                        }
                    ]
                }
            ]
        }
    }

import json
print(json.dumps(config, indent=4))