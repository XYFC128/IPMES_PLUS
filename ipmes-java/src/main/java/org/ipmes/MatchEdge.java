package org.ipmes;

public class MatchEdge {
    Integer eventId;
    public Integer getEventId() {
        return this.eventId;
    }

    Integer timestamp;
    public Integer getTimestamp() {
        return this.timestamp;
    }

    Integer startId;
    public Integer getStartId() {
        return this.startId;
    }

    Integer endId;
    public Integer getEndId() {
        return this.endId;
    }

    Integer patterId;
    public Integer getPatternId() {
        return this.patterId;
    }

    MatchEdge(Integer eventId, Integer timestamp, Integer startId, Integer endId, Integer patternId) {
        this.eventId = eventId;
        this.timestamp = timestamp;
        this.startId = startId;
        this.endId = endId;
        this.patterId = patternId;
    }

    public Integer[] getEndponts() {
        return new Integer[] {this.startId, this.endId};
    }
}
