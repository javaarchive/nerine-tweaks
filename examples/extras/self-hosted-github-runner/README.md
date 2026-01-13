# Self-hosted GitHub runner

**WARNING: This is not a supported configuration. We assume you trust anyone with write access to your challenges repository.**

Instead of setting up a container registry and using a hosted GitHub Actions runner to push to it, you can use a self-hosted GitHub runner that builds images on the challenges machine.

## Installation
1. Fill in the `.env` from `.env.example`.
2. Setup the correct docker group for the container user in the bottom commented section of the `docker-compose.yml` file.
3. Run `docker-compose up -d`. You may wish to vary the work directory.