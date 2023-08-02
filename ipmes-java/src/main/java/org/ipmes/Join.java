package org.ipmes;

import java.util.ArrayList;
import java.util.Map;

import java.util.HashMap;

public class Join {
    DependencyGraph temporalRelation;
    PatternGraph spatialRelation;
    ArrayList<Map<Integer, MatchEdge>> answer;
    ArrayList<Map<Integer, MatchEdge>> expansionTable;
    Map<Integer, ArrayList<TCQueryRelation>> TCQRelation;

    public Join(DependencyGraph temporalRelation, PatternGraph spatialRelation,
            Map<Integer, ArrayList<TCQueryRelation>> TCQRelation) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
        this.answer = new ArrayList<Map<Integer, MatchEdge>>();
        this.expansionTable = new ArrayList<Map<Integer, MatchEdge>>();
        this.TCQRelation = TCQRelation;
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

    private boolean checkRelation(MatchEdge edgeInMatchResult, MatchEdge edgeInTable) {
        Integer[][] arr = {
                edgeInMatchResult.matched.getEndpoints(),
                edgeInTable.matched.getEndpoints(),
                edgeInMatchResult.getEndpoints(),
                edgeInTable.getEndpoints()
        };
        return relationType(arr[0], arr[1]) == relationType(arr[2], arr[3]);
    }

    private boolean checkTime(MatchEdge edgeInMatchResult, MatchEdge edgeInTable) {
        return (this.temporalRelation.getParents(edgeInMatchResult.matched.getId())
                .contains(edgeInTable.matched.getId())
                && edgeInMatchResult.timestamp >= edgeInTable.timestamp)
                ||
                (this.temporalRelation.getChildren(edgeInMatchResult.matched.getId())
                        .contains(edgeInTable.matched.getId())
                        && edgeInMatchResult.timestamp <= edgeInTable.timestamp);
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

    boolean checkNoOverlap(Map<Integer, MatchEdge> entry, ArrayList<MatchEdge> result) {
        for (MatchEdge edge : result) {
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
    void combineResult(Map<Integer, MatchEdge> combineTo, ArrayList<MatchEdge> result) {
        for (MatchEdge edge : result) {
            combineTo.put(edge.matched.getId(), edge);
        }
        if (combineTo.size() == this.spatialRelation.numEdges())
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
    public void addMatchResult(ArrayList<MatchEdge> result, Integer tcQueryId) {
        boolean fit = true;
        ArrayList<Map<Integer, MatchEdge>> buffer = new ArrayList<Map<Integer, MatchEdge>>();
        // join
        for (Map<Integer, MatchEdge> entry : this.expansionTable) {
            if (checkNoOverlap(entry, result)) {
                for (TCQueryRelation relationship : this.TCQRelation.get(tcQueryId)) {
                    if (entry.containsKey(relationship.idOfEntry)) {
                        for (MatchEdge tmpEdge : result) {
                            if (tmpEdge.matched.getId() != relationship.idOfResult)
                                continue;
                            if (!(checkRelation(tmpEdge, entry.get(relationship.idOfEntry))
                                    && checkTime(tmpEdge, entry.get(relationship.idOfEntry)))) {
                                fit = false;
                                break;
                            }
                        }
                        if (!fit)
                            break;
                    }
                }

                // for (Map.Entry<Integer, MatchEdge> entryInTable : entry.entrySet()) {
                // MatchEdge edgeInTable = entryInTable.getValue();
                // for (MatchEdge edgeInResult : result) {
                // if (!(checkRelation(edgeInResult, edgeInTable)
                // && checkTime(edgeInResult, edgeInTable))) {
                // fit = false;
                // break;
                // }
                // }
                // if (!fit)
                // break;
                // }
                if (fit) {
                    Map<Integer, MatchEdge> temp = new HashMap<Integer, MatchEdge>(entry);
                    combineResult(temp, result);
                    buffer.add(temp);
                }
            }
            fit = true;
        }
        // insert
        Map<Integer, MatchEdge> temp = new HashMap<Integer, MatchEdge>();
        combineResult(temp, result);
        this.expansionTable.add(temp);
        this.expansionTable.addAll(buffer);
        buffer.clear();
    }

    public ArrayList<ArrayList<MatchEdge>> extractAnswer() {
        ArrayList<ArrayList<MatchEdge>> ret = new ArrayList<ArrayList<MatchEdge>>();
        int len = this.spatialRelation.numEdges();
        for (Map<Integer, MatchEdge> entry : this.answer) {
            ArrayList<MatchEdge> temp = new ArrayList<MatchEdge>();
            for (int i = 0; i < len; i++) {
                temp.add(entry.get(i));
            }
            ret.add(temp);
        }
        this.answer.clear();
        return ret;
    }
}
