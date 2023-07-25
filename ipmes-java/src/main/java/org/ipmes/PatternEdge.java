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

    @Override
    public String toString() {
        return String.format(
                "Edge {id: %d, signature: %s, startId: %d, endId: %d}",
                this.id,
                this.signature,
                this.startId,
                this.endId
        );
    }

    @Override
    public boolean equals(Object other) {
        if (other == this) return true;
        if (!(other instanceof PatternEdge)) return false;
        PatternEdge temp = (PatternEdge) other;
        return this.id.equals(temp.id) && this.signature.equals(temp.signature) &&
                this.startId.equals(temp.startId) && this.endId.equals(temp.endId);
    }
}
