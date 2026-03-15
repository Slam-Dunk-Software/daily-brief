# Customizing daily-brief

daily-brief texts you a summary of your open tasks at a scheduled time every
day. It connects to a `todo` EPS for tasks and `txtme` for delivery.

---

## Ports

### `MESSAGE_FORMAT` — brief content
**Type:** source-level edit
**Location:** `send_brief()` in `src/main.rs`

The function that builds the message string. Edit it to change what gets
included, how tasks are formatted, or to pull from additional sources.

Default format:
```
GM!
Next up: Buy milk
```

---

### `TODO_URL` — task source
**Type:** environment variable
**Default:** `http://localhost:8765`

URL of a `todo` EPS instance. daily-brief calls `GET {TODO_URL}/tasks` and
expects a JSON array of `{"text": "...", "done": false}`.

---

### `TXTME_URL` — notification sink
**Type:** environment variable
**Default:** `http://localhost:5543`

URL of a `txtme` instance. daily-brief `POST`s the message here.

---

### `TXTME_API_KEY` — txtme auth
**Type:** environment variable
**Default:** none

If your txtme instance has `TXTME_API_KEY` set, provide the same value here.

---

### `BRIEF_HOUR` — time of day
**Type:** environment variable
**Default:** `7` (7 AM)

Hour of day (0–23) to send the brief, in the timezone set by `BRIEF_TZ`.

---

### `BRIEF_TZ` — timezone
**Type:** environment variable
**Default:** `America/New_York`

Any IANA timezone name (e.g. `America/Los_Angeles`, `Europe/London`,
`UTC`). Controls what "hour" means for `BRIEF_HOUR`.

---

### `HOST` — bind address
**Type:** environment variable
**Default:** `127.0.0.1`

---

### `PORT` — port number
**Type:** environment variable
**Default:** `5544`
