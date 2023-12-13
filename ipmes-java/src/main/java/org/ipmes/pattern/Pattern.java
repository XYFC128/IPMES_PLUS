package org.ipmes.pattern;

public class Pattern {
    public boolean useRegex;
    public PatternGraph patternGraph;
    public TemporalRelation temporalRelation;

    Pattern(PatternGraph graph, TemporalRelation relation) {
        this.patternGraph = graph;
        this.temporalRelation = relation;
    }
}
