import sys
import os

def bail(message: str):
    sys.stderr.write(message + "\n")
    sys.stderr.flush()
    sys.exit(1)

def env_boolean(name: str) -> bool:
    if name in os.environ:
        return os.environ[name].lower() in ("1", "true", "yes")
    return False

ENABLE_HTTPS_PLATFORM = env_boolean("ENABLE_HTTPS_PLATFORM")
# you should not need this dns challenges config tbh
ENABLE_CF_DNS_CHALLENGES = env_boolean("ENABLE_CF_DNS_CHALLENGES")

if len(sys.argv) != 2:
    bail("Usage: python3 generate_simple_caddyfile_config.py <platform-domain>" + str(sys.argv))

main_filename, platform_domain = sys.argv

if platform_domain.endswith(".localhost") and ENABLE_HTTPS_PLATFORM:
    bail("You cannot use .localhost with ENABLE_HTTPS_PLATFORM set to true")

matcher = f"{platform_domain}" if ENABLE_HTTPS_PLATFORM else f"http://{platform_domain}"

print(f"""
{matcher} {{
        reverse_proxy /api/* api:3333
        reverse_proxy /* frontend:3334

        log {{
                output file /var/log/caddy/access.log {{
                        roll_size 1gb
                        roll_keep 20
                        roll_keep_for 720h
                }}
        }}

        # add cloudflare config here if needed (prob don't)...
}}
""")