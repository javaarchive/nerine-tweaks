import os
import sys
import secrets

def bail(message: str):
    sys.stderr.write(message + "\n")
    sys.stderr.flush()
    sys.exit(1)

def env_boolean(name: str) -> bool:
    if name in os.environ:
        return os.environ[name].lower() in ("1", "true", "yes")

db_password = secrets.token_urlsafe(32)
admin_token = secrets.token_urlsafe(32)
jwt_secret = secrets.token_urlsafe(32)

platform_domain = os.environ.get("PLATFORM_DOMAIN", None)
enable_https_platform = env_boolean("ENABLE_HTTPS_PLATFORM")
platform_url_protocol = "https" if enable_https_platform else "http"

with open(".env.example", "r") as f:
    for line in f:
        if not "=" in line:
            print(line.strip())
            continue
        key, value = line.split("=")
        if key == "DATABASE_URL":
            print(f"DATABASE_URL=postgres://nerine:{db_password}@db/nerine")
        elif key == "POSTGRES_PASSWORD":
            print(f"POSTGRES_PASSWORD={db_password}")
        elif key == "ADMIN_TOKEN":
            print(f"ADMIN_TOKEN={admin_token}")
        elif key == "JWT_SECRET":
            print(f"JWT_SECRET={jwt_secret}")
        elif key == "CORS_ORIGIN" and platform_domain:
            print(f"CORS_ORIGIN={platform_url_protocol}://{platform_domain}")
        else:
            print(line.strip())