package org.ipmes.pattern;

public class PatternEdge {
    Integer id;
    public Integer getId() { return this.id; }

    String signature;
    public String getSignature() { return this.signature; }

    PatternNode startNode;
    public PatternNode getStartNode() { return this.startNode; }
    public Integer getStartId() { return this.startNode.getId(); }

    PatternNode endNode;
    public PatternNode getEndNode() { return this.endNode; }
    public Integer getEndId() { return this.endNode.getId(); }

    public PatternEdge(Integer id, String signature, PatternNode startNode, PatternNode endNode) {
        this.id = id;
        this.signature = String.format("%s#%s#%s", signature, startNode.signature, endNode.signature);
        this.startNode = startNode;
        this.endNode = endNode;
    }

    public Integer[] getEndpoints() {
        return new Integer[] {this.getStartId(), this.getEndId()};
    }

    @Override
    public String toString() {
        return String.format(
                "PatternEdge {id: %d, signature: %s, startId: %d, endId: %d}",
                this.id,
                this.signature,
                this.getStartId(),
                this.getEndId()
        );
    }

    @Override
    public boolean equals(Object other) {
        if (other == this) return true;
        if (!(other instanceof PatternEdge)) return false;
        PatternEdge temp = (PatternEdge) other;
        return this.id.equals(temp.id) && this.signature.equals(temp.signature) &&
                this.startNode.equals(temp.startNode) && this.endNode.equals(temp.endNode);
    }
}
