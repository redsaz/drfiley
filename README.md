# DrFiley

View, organize, and deduplicate years of files across all your machines.


# Configuration

DrFiley has two components:

- The Agent: installed on every machine you wish to monitor and manage
- The Server: Collects the information from every Agent. Connect to it from your browser to view your files.

This will walk you through how to configure both.

## Agent

The agent will use the following config by default:

```toml
debug = true
max_threads = 1
path = "."
```

These can be configured via environment variables or via a `.env` file in the working directory:

```bash
# .env file:
DRFILEY_AGENT_DEBUG=true
DRFILEY_AGENT_MAX_THREADS=1
DRFILEY_AGENT_PATH=/home/exampleuser/Downloads
```

```bash
# Or, commandline:
DRFILEY_AGENT_PATH=/home/exampleuser/Downloads; ./drfiley-agent
```

The Agent will also look for `drfiley-agent.toml` in the working directory, which can look like this:

```toml
debug = true
max_threads = 1
path = /home/exampleuser/Downloads
```

If `DRFILEY_AGENT_CONFIG` is an environment variable or in the `.env` file, it will look for a configuration there:

```bash
DRFILEY_AGENT_CONFIG=/etc/drfiley/agent.toml; ./drfiley-agent
```

The configuration will be built from any and all of these sources. If a value is specified in multiple locations, it will be prioritized as follows (lowest to highest):

- defaults
- `drfiley-agent.toml`
- custom toml file
- environment variables and `.env`


## Server

There is no server yet.


# Notes

To build in Fedora, protoc and protobuf-static, protobuf-devel packages need installed for well-defined types
