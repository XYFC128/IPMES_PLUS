package org.ipmes.pattern;

/**
 * A node in a pattern. Typically accessed through {@link PatternEdge} or {@link PatternGraph}
 */
public class PatternNode {
    Integer id;
    public Integer getId() { return this.id; }

    String signature;
    public String getSignature() { return this.signature; }

    public PatternNode(Integer id, String signature) {
        this.id = id;
        this.signature = signature;
    }

    @Override
    public String toString() {
        return String.format(
                "Node {id: %d, signature: %s}",
                this.id,
                this.signature
        );
    }

    @Override
    public boolean equals(Object other) {
        if (other == this) return true;
        if (!(other instanceof PatternNode)) return false;
        PatternNode temp = (PatternNode) other;
        return this.id.equals(temp.id) && this.signature.equals(temp.signature);
    }
}
