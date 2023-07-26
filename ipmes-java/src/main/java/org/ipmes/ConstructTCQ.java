package org.ipmes;

import java.io.FileNotFoundException;
import java.io.FileReader;
import java.util.*;

public class ConstructTCQ {

    public static void main(String[] args) throws FileNotFoundException {

        /*
         * ///////////////////////////////////////////////
         * initialization
         *////////////////////////////////////////////////

        // parsing temporal relationship
        FileReader orelsReader = new FileReader("/Users/datou/repos/CITI/data/patterns/TTP10_oRels.json");
        Optional<DependencyGraph> tmpTemporalRelation = DependencyGraph.parse(orelsReader);
        DependencyGraph TemporalRelation = tmpTemporalRelation.get();

        // parsing spatial relationship
        FileReader nodeReader = new FileReader("/Users/datou/repos/CITI/data/patterns/TTP10_node.json");
        FileReader edgeReader = new FileReader("/Users/datou/repos/CITI/data/patterns/TTP10_edge.json");
        Optional<PatternGraph> tmpSpatialRelation = PatternGraph.parse(nodeReader, edgeReader);
        PatternGraph SpatialRelation = tmpSpatialRelation.get();

        /*
         * ///////////////////////////////////////////////
         * DFS to generate TC subqueries
         *////////////////////////////////////////////////

        int numEdges = SpatialRelation.getEdges().size();
        ArrayList<TCQuery> subQueries = new ArrayList<TCQuery>();
        for (int i = 0; i < numEdges; i++) {
            ArrayList<Integer> tmpList = new ArrayList<Integer>();
            tmpList.add(i);
            subQueries.addAll(TCQuery.generate_TCQueries(TemporalRelation, SpatialRelation, tmpList, i));
        }

        /*
         * ///////////////////////////////////////////////
         * greedy select longest TC subqueries
         *////////////////////////////////////////////////

        // sort in decreasing size
        subQueries.sort((Q1, Q2) -> (Q2.query.size() - Q1.query.size()));
        // subQ_provider: index of subqueries
        ArrayList<Integer> subQ_provider = new ArrayList<Integer>();
        // subQ_selected: result of greedy select
        ArrayList<Integer> subQ_selected = new ArrayList<Integer>();
        for (int i = 0; i < subQueries.size(); i++) {
            boolean shouldSelect = true;
            subQueries.get(i).setTCQueryID(i);
            // System.out.println(subQueries.get(i).getTCQueryID() + ": " +
            // subQueries.get(i).getQueryEdges());
            for (int j : subQueries.get(i).query) {
                if (subQ_selected.contains(j)) {
                    shouldSelect = false;
                    break;
                }
            }
            if (shouldSelect) {
                subQ_provider.add(i);
                subQ_selected.addAll(subQueries.get(i).query);
            }
        }

        /*
         * ///////////////////////////////////////////////
         * generate Siddhi Query from TC subqueries
         *////////////////////////////////////////////////
    }
}
