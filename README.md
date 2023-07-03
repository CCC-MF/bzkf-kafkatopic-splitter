# BZKF Kafkatopic Splitter

Anwendung zum Neugruppieren von Nachrichten basierend auf Angaben zum Jahr einer Meldung.

## Docker-Image

Das zugehörige Docker-Image kann über die folgenden Parameter konfiguriert werden:

* `KAFKA_GROUP_ID`: Consumer-Gruppe, Standardwert wenn nicht angegeben: `bzkf-kafkatopic-splitter`
* `KAFKA_BOOTSTRAP_SERVERS`: Kafka-Bootstrapserver mit Standardwert: `kafka:9094`
* `KAFKA_SOURCE_TOPIC`: Quell-Topic, aus dem Nachrichten gelesen werden. Standardwert: `onkostar.MELDUNG_EXPORT`
* `KAFKA_DESTINATION_TOPIC_PREFIX`: Prefix für die Zieltopics. Standardwert: `onkostar.MELDUNG_EXPORT.` (mit Punkt am Ende). Hier wird das Jahr angehängt.

Container werden, wie auch andere BZKF-Container, als User/Gruppe 65532:65532 ausgeführt.

Kann keine Verbindung zu Kafka aufgebaut werden, beendet sich der Container und kann durch `restart: unless-stopped` automatisch neu gestartet werden.

## Erforderliche Inhalte der Nachrichten

Key und Nachrichteninhalt werden aus der Quellnachricht kopiert.
Der Nachrichteninhalt muss als JSON vorliegen und die Jahresangabe als `YEAR` angeben. 