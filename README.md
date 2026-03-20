This is a Rust CLI tool for spreadsheet-driven API testing with image payloads.

It reads test cases from an Excel file (`Sheet1`), loads images from a folder, base64-encodes them, and sends POST requests to `<address><url>` for each row.

For each request, it builds a JSON payload like:
- `request_name` from the `param` column
- `images` as a one-item array containing the encoded image
- `params` from the `args` column

The `args` column can be either JSON or a Go-style `map[...]` string, which is parsed into JSON before sending.

Responses are checked against the expected substring in the `response` column. Any mismatch (or request failure) is appended to an output text file with the row number.

CLI options:
- `--address <URL>`: base server address
- `--excel <FILE>`: input `.xlsx` file
- `--folder <FOLDER>`: image folder (default: `./images`)
- `--output <FILE>`: mismatch output file (default: `output.txt`)
