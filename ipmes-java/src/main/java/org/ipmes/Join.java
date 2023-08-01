package org.ipmes;

import java.util.ArrayList;
import java.util.Map;

import javax.xml.crypto.Data;

import java.util.HashMap;

public class Join {
    DependencyGraph temporalRelation;
    PatternGraph spatialRelation;
    ArrayList<Map<Integer, DataEdge>> answer;
    ArrayList<Map<Integer, DataEdge>> expansionTable;

    public Join(DependencyGraph temporalRelation, PatternGraph spatialRelation) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
        this.answer = new ArrayList<Map<Integer, DataEdge>>();
        this.expansionTable = new ArrayList<Map<Integer, DataEdge>>();
    }

    /**
     * Use bit-operation like method to check edge spatial relation
     * <p>
     * we have two input DataEdge DE1 and DE2,
     * if the relation between DE1.matched.getEndPoints and DE2.matched.getEndPoints
     * equals DE1.getEndPoints and DE2.getEndPoints, return true.
     * Otherwise, return false.
     * </p>
     * 
     * @param n endpoints of one edge
     * @param m endpoints of another edge
     * @return the relationship type
     */
    private byte relationType(Integer n[], Integer m[]) {
        byte ret = 0;
        for (int i : n) {
            for (int j : m) {
                if (i == j)
                    ret |= 1;
                ret <<= 1;
            }
        }
        return ret;
    }

    private boolean checkRelation(DataEdge edgeInMatchResult, DataEdge edgeInTable) {
        Integer arr[][] = {
                edgeInMatchResult.matched.getEndpoints(),
                edgeInTable.matched.getEndpoints(),
                edgeInMatchResult.getEndpoints(),
                edgeInTable.getEndpoints()
        };
        return relationType(arr[0], arr[1]) == relationType(arr[2], arr[3]);
    }

    private boolean checkTime(DataEdge edgeInMatchResult, DataEdge edgeInTable) {
        return (this.temporalRelation.getParents(edgeInMatchResult.matched.getId())
                .contains(edgeInTable.matched.getId())
                && edgeInMatchResult.timestamp > edgeInTable.timestamp)
                ||
                (this.temporalRelation.getChildren(edgeInMatchResult.matched.getId())
                        .contains(edgeInTable.matched.getId())
                        && edgeInMatchResult.timestamp < edgeInTable.timestamp)
                ||
                (!this.temporalRelation.getParents(edgeInMatchResult.matched.getId())
                        .contains(edgeInTable.matched.getId())
                        && !this.temporalRelation.getChildren(edgeInMatchResult.matched.getId())
                                .contains(edgeInTable.matched.getId()));
    }

    /**
     * save match result of TC sub-queries
     * <p>
     * When we want to join the match result, we check the following constraints:
     * <ol>
     * <li>the two match results do not overlap</li>
     * <li>the spatial relation of the two match results are fine</li>
     * <li>the temporal relation of the two match results are fine</li>
     * </ol>
     * If all constraints are followed, create a new entry and add it to
     * expansionTable.
     * If the entry contains every edge, it is one of the match results of the whole
     * pattern.
     * <\p>
     * 
     * @return the match results of the whole pattern
     */

    /**
     * Detect whether any edge in subTCQ appear in entry
     * 
     * @param entry  the matched entry
     * @param result the match result of TC subquery
     * @return true if no overlapping between entry and result
     */

    boolean checkNoOverlap(Map<Integer, DataEdge> entry, ArrayList<DataEdge> result) {
        for (DataEdge edge : result) {
            if (entry.containsKey(edge.matched.getId()))
                return false;
        }
        return true;
    }

    /**
     * add match result to the Map combineTo, and add combineTo to expansionTable.
     * 
     * @param combineTo the Map we want to add result to
     * @param result    the match result of TC subquery
     * 
     */
    void combineResult(Map<Integer, DataEdge> combineTo, ArrayList<DataEdge> result) {
        for (DataEdge edge : result) {
            combineTo.put(edge.matched.getId(), edge);
        }
        this.expansionTable.add(combineTo);
        if (combineTo.size() == this.spatialRelation.getEdges().size())
            this.answer.add(combineTo);
        return;
    }

    /**
     * Consume the match result and start the streaming join algorithm.
     * <p>
     * TODO: preprocess which edges' relationships need to be checked.
     * </p>
     * <p>
     * When we want to join the match result, we check the following constraints:
     * <ol>
     * <li>the two match results do not overlap</li>
     * <li>the spatial relation of the two match results are fine</li>
     * <li>the temporal relation of the two match results are fine</li>
     * </ol>
     * If all constraints are followed, create a new entry and add it to
     * expansionTable.
     * If the entry contains every edge, it is one of the match results of the whole
     * pattern.
     * <\p>
     * 
     * @param result    the match result
     * @param tcQueryId the TC-Query id of the result
     */
    public void addMatchResult(ArrayList<DataEdge> result, Integer tcQueryId) {
        boolean fit = true;
        // join
        for (Map<Integer, DataEdge> entry : this.expansionTable) {
            if (checkNoOverlap(entry, result)) {
                for (Map.Entry<Integer, DataEdge> entryInTable : entry.entrySet()) {
                    for (DataEdge edgeInResult : result) {
                        DataEdge edgeInTable = entryInTable.getValue();
                        if (!(checkRelation(edgeInResult, edgeInTable)
                                && checkTime(edgeInResult, edgeInTable))) {
                            fit = false;
                            break;
                        }
                    }
                    if (!fit)
                        break;
                }
                if (fit) {
                    Map<Integer, DataEdge> temp = new HashMap<Integer, DataEdge>(entry);
                    combineResult(temp, result);
                }
            }
            fit = true;
        }
        // insert
        Map<Integer, DataEdge> temp = new HashMap<Integer, DataEdge>();
        combineResult(temp, result);
    }

    public ArrayList<ArrayList<DataEdge>> extractAnswer() {
        ArrayList<ArrayList<DataEdge>> ret = new ArrayList<ArrayList<DataEdge>>();
        ArrayList<DataEdge> temp = new ArrayList<DataEdge>();
        int len = this.spatialRelation.getEdges().size();
        for (Map<Integer, DataEdge> entry : this.answer) {
            for (int i = 0; i < len; i++) {
                temp.add(entry.get(i));
            }
            ret.add(temp);
            temp.clear();
        }
        this.answer.clear();
        return ret;
    }
}
