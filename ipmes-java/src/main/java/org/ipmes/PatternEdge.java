package org.ipmes;

public class PatternEdge {
    Integer id;
    String signature;
    Integer startId;
    Integer endId;
    public PatternEdge(Integer id, String signature, Integer startId, Integer endId) {
        this.id = id;
        this.signature = signature;
        this.startId = startId;
        this.endId = endId;
    }

    public String toString() {
        return String.format(
                "Edge {id: %d, signature: %s, startId: %d, endId: %d}",
                this.id,
                this.signature,
                this.startId,
                this.endId
        );
    }
}
