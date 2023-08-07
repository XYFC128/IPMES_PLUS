package org.ipmes;

import org.ipmes.pattern.Preprocess;
import org.json.JSONObject;

/**
 * EventEdge represent an event about to be fed into CEP tool
 */
public class EventEdge {
    public String timestamp;
    public String edgeId;
    public String signature;
    public String startId;
    public String endId;

    public EventEdge(String csvRow) {
        String[] fields = csvRow.split(",");
        this.timestamp = fields[0];
        this.signature = fields[1];
        this.edgeId    = fields[2];
        this.startId   = fields[3];
        this.endId     = fields[4];
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
}
