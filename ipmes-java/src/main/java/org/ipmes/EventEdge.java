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

    public EventEdge(long timestamp, String signature, long edgeId, long startId, long endId) {
        this.timestamp = timestamp;
        this.signature = signature;
        this.edgeId    = edgeId;
        this.startId   = startId;
        this.endId     = endId;
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
