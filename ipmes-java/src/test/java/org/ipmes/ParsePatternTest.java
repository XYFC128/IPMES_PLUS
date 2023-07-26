package org.ipmes;

import org.junit.Test;

import java.io.StringReader;
import java.util.Optional;

import static org.junit.Assert.*;

public class ParsePatternTest {
    @Test
    public void normalParsing() {
        String node = TTPGenerator.genTTP11Nodes();
        String edge = TTPGenerator.genTTP11Edges();

        StringReader nodeReader = new StringReader(node);
        StringReader edgeReader = new StringReader(edge);
        Optional<PatternGraph> res = PatternGraph.parse(nodeReader, edgeReader);
        assertTrue(res.isPresent());

        PatternGraph g = res.get();

        PatternNode[] nodeAns = {
                new PatternNode(0, "Process::hello.sh"),
                new PatternNode(1, "Process::hello.sh"),
                new PatternNode(2, "Process::journalctl"),
                new PatternNode(3, "Artifact::directory::/var/log/journal"),
                new PatternNode(4, "Artifact::file::/usr/bin/journalctl")
        };
        assertArrayEquals(nodeAns, g.getNodes().toArray());

        PatternEdge[] edgeAns = {
                new PatternEdge(0, "fork", 0, 1),
                new PatternEdge(1, "execve", 1, 2),
                new PatternEdge(2, "load", 4, 2),
                new PatternEdge(3, "open", 3, 2),
        };
        assertArrayEquals(edgeAns, g.getEdges().toArray());

        // shared nodes
        assertTrue(g.getSharedNodes(0, 3).isEmpty());
        assertArrayEquals(
                new Integer[] {2},
                g.getSharedNodes(1, 2).toArray()
        );
    }

    @Test
    public void errorParsing() {
        String node = "aaa";
        String edge = "";
        StringReader nodeReader = new StringReader(node);
        StringReader edgeReader = new StringReader(edge);
        Optional<PatternGraph> res = PatternGraph.parse(nodeReader, edgeReader);
        assertFalse(res.isPresent());
    }
}
