# gnome-pomodoro-watcher

A helper tool to watch GNOME Pomodoro timer.

## How it works

gnome-pomodoro-watcher watches GNOME Pomodoro timer and continuously outputs
its state in JSON format when it changes.

```
{"remaining_secs":0,"is_paused":false,"state":"stopped"}
{"remaining_secs":1500,"is_paused":false,"state":"pomodoro"}
{"remaining_secs":1499,"is_paused":false,"state":"pomodoro"}
{"remaining_secs":1498,"is_paused":false,"state":"pomodoro"}
{"remaining_secs":1497,"is_paused":false,"state":"pomodoro"}
{"remaining_secs":1496,"is_paused":false,"state":"pomodoro"}
{"remaining_secs":1495,"is_paused":false,"state":"pomodoro"}
{"remaining_secs":1495,"is_paused":true,"state":"pomodoro"}
```

| Field            | Type      | Description                                                                                                           |
| ---------------- | --------- | --------------------------------------------------------------------------------------------------------------------- |
| `remaining_secs` | `number`  | Remaining time of the current timer in seconds.                                                                       |
| `is_paused`      | `boolean` | Whether the timer is paused.                                                                                          |
| `state`          | `string`  | Current timer state. One of `stopped`, `pomodoro`, `short-break`, `long-break`. Note that there is no `paused` state. |

## Usage

You can easily format the timer state with [jq](https://jqlang.github.io/jq/).

```bash
#!/usr/bin/env bash

gnome-pomodoro-watcher | jq -r '.remaining_secs | strftime("%M:%S")'
```

And you can display it on [Waybar](https://github.com/Alexays/Waybar) with [Custom](https://github.com/Alexays/Waybar/wiki/Module:-Custom) module.

```json
{
  "custom/gnome-pomodoro": {
    "exec": "/path/to/custom-script.bash",
    "interval": 1
  }
}
```
