# How to set Channel Settings
At the root of your soundpack directory, create an xml file.
While the name can be arbitrary, `channelSettings.xml` is recommended.

This file should contain one `channelSettings`, which contain multiple `channelSetting`.

| Attribute | Possible Values               | Description    |
| --        | --                            | -----------    |
| name      | _channel_name_ (__required__) | Channel's name.<br>Setting for any channels that are not in the soundpack will be ignored. |
| playType  | "all"(_default_), "singleEager", "singleLazy" | How the channel will play sounds.<br>__"all"__: will play all sounds. <br>__"singleEager"__: will play one sound at a time, and will pause/stop the current playing sounds when a new sound is triggered. <br>__"singleLazy"__: will play one sound at a time, and will ignore new sounds when already playing a sound. |

Example:
```
<?xml version="1.1" encoding="UTF-8"?>
<channelSettings>
	<channelSetting name="music" playType="singleEager"/>
	<channelSetting name="weather" playType="singleEager"/>
	<channelSetting name="trade" playType="singleLazy"/>
</channelSettings>
```

Attributes may be added/changed in the future.