package org.ipmes;

public class PatternNode {
    Integer id;
    String signature;
    public PatternNode(Integer id, String signature) {
        this.id = id;
        this.signature = signature;
    }

    public String toString() {
        return String.format(
                "Node {id: %d, signature: %s}",
                this.id,
                this.signature
        );
    }
}
