package org.ipmes;

import org.junit.Test;

import java.io.StringReader;
import java.util.Optional;

import static org.junit.Assert.*;

public class ParseDependencyTest {
    @Test
    public void parseTemporalRelation() {
        String orels = TTPGenerator.genTTP11Orels();
        Optional<DependencyGraph> res = DependencyGraph.parse(new StringReader(orels));
        assertTrue(res.isPresent());
        DependencyGraph g = res.get();

        assertArrayEquals(
                new Integer[] {},
                g.getParents(-1).toArray());
        assertArrayEquals(
                new Integer[] {-1},
                g.getParents(0).toArray());
        assertArrayEquals(
                new Integer[] {0},
                g.getParents(1).toArray());
        assertArrayEquals(
                new Integer[] {1},
                g.getParents(2).toArray());
        assertArrayEquals(
                new Integer[] {2},
                g.getParents(3).toArray());

        assertArrayEquals(
                new Integer[] {0},
                g.getChildren(-1).toArray());
        assertArrayEquals(
                new Integer[] {1},
                g.getChildren(0).toArray());
        assertArrayEquals(
                new Integer[] {2},
                g.getChildren(1).toArray());
        assertArrayEquals(
                new Integer[] {3},
                g.getChildren(2).toArray());
        assertArrayEquals(
                new Integer[] {},
                g.getChildren(3).toArray());
    }
}
