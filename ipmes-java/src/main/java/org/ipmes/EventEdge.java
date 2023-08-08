package org.ipmes;

import org.ipmes.pattern.Preprocess;
import org.json.JSONObject;

/**
 * EventEdge represent an event about to be fed into CEP tool
 */
public class EventEdge {
    public long timestamp;
    public long edgeId;
    public String signature;
    public long startId;
    public long endId;

    static long parseTimestamp(String tsStr) {
        double toNum = Double.parseDouble(tsStr);
        return Math.round(toNum * 1000);
    }

    public EventEdge(String csvRow) {
        String[] fields = csvRow.split(",");
        this.timestamp = parseTimestamp(fields[0]);
        this.signature = fields[1];
        this.edgeId    = Long.parseLong(fields[2]);
        this.startId   = Long.parseLong(fields[3]);
        this.endId     = Long.parseLong(fields[4]);
    }

    public EventEdge(EventEdge other) {
        this.timestamp = other.timestamp;
        this.signature = other.signature;
        this.edgeId    = other.edgeId;
        this.startId   = other.startId;
        this.endId     = other.endId;
    }

    public Object[] toEventData() {
        return new Object[]{timestamp, edgeId, startId, endId};
    }

    @Override
    public String toString() {
        return String.format("EventEdge <%s> {sig: %s, id: %s}", this.timestamp, this.signature, this.edgeId);
    }
}
