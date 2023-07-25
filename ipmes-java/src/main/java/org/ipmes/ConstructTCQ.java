package main.java.org.ipmes;

import java.io.FileReader;

import org.ipmes.DependencyGraph;
import org.ipmes.PatternGraph;

public class ConstructTCQ {
    public static void main(String[] args) {

        /*
         * //////////////////////////////////
         * Initialization
         *///////////////////////////////////

        // parsing temporal relationship
        FileReader orelsReader = new FileReader(args[0]);
        Optional<DependencyGraph> tmpTemporalRelation = DependencyGraph.parse(orelsReader);
        DependencyGraph TemporalRelation = tmpTemporalRelation.get();

        // parsing spatial relationship
        FileReader nodeReader = new FileReader(args[1]);
        FileReader edgeReader = new FileReader(args[2]);
        Optional<PatternGraph> tmpSpatialRelation = PatternGraph.parse(nodeReader, edgeReader);
        PatternGraph SpatialRelation = tmpSpatialRelation.get();

    }
}
