package org.ipmes;

import org.junit.Test;

import java.io.StringReader;
import java.util.Optional;

import static org.junit.Assert.*;

public class ParseDependencyTest {
    @Test
    public void parseTemporalRelation() {
        String orels = "{\"0\":{\"parents\":[\"root\"],\"children\":[1]},\"1\":{\"parents\":[0],\"children\":[2]},\"2\":{\"parents\":[1],\"children\":[3]},\"3\":{\"parents\":[2],\"children\":[]},\"root\":{\"parents\":[],\"children\":[0]}}";
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
                g.getChildrens(-1).toArray());
        assertArrayEquals(
                new Integer[] {1},
                g.getChildrens(0).toArray());
        assertArrayEquals(
                new Integer[] {2},
                g.getChildrens(1).toArray());
        assertArrayEquals(
                new Integer[] {3},
                g.getChildrens(2).toArray());
        assertArrayEquals(
                new Integer[] {},
                g.getChildrens(3).toArray());
    }
}
