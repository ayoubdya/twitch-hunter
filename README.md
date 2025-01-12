# Twitch Hunter

Twitch Hunter is a tool that connects to multiple Twitch IRC channels and looks for specific regex patterns in chat messages. It uses the Twitch Helix API to fetch streams from a specific category and then monitors the chat messages in those streams.

## Features

- Connects to multiple Twitch IRC channels.
- Filters chat messages based on a regex pattern.
- Uses the Twitch Helix API to fetch streams from a specific category.

## Installation

1. Clone the repository:

   ```sh
   git clone https://github.com/ayoubdya/twitch-hunter.git
   cd twitch-hunter
   ```

2. Install dependencies:
   ```sh
   cargo build
   ```

## Configuration

Set the following environment variables:

- `TWITCH_CLIENT_ID`: Your Twitch client ID.
- `TWITCH_ACCESS_TOKEN`: Your Twitch access token.

You can set these variables in your shell or create a `.env` file in the project root:

```sh
TWITCH_CLIENT_ID=your_client_id
TWITCH_ACCESS_TOKEN=your_access_token
```

## Usage

Run the application:

```sh
cargo run
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## Authors

- Ayoub DYA - [ayoubdya@gmail.com](mailto:ayoubdya@gmail.com)
