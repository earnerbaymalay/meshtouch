# MeshRelay troubleshooting guide

Common issues and solutions.

---

## Relay daemon issues

### Relay server not starting
- Ensure Rust and its dependencies are correctly installed.
- Check `config.toml` for correct configuration.
- Verify that port 8080 is not in use by another application.
- Review server logs for specific error messages.

### Messages not being stored or fetched
- Check database permissions for the SQLite file.
- Verify the relay server is running without errors.
- Ensure correct API endpoints are being used (`/api/v1/messages`).

---

## Web reader issues

### Messages not decrypting in browser
- Verify the `key` and `iv` parameters in the URL fragment are correct and uncorrupted.
- Ensure your browser supports Web Crypto API.
- Check browser console for JavaScript errors related to decryption.

### Web reader link invalid
- Confirm the `message_id` in the URL is correct.
- Ensure the relay server is online and the message exists.

---

## General issues

### Low message throughput
- For Raspberry Pi relays, consider using a more powerful device or optimizing the Rust build.
- Review server logs for any performance bottlenecks.

---

[MIT License](LICENSE)
