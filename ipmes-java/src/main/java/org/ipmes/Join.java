package org.ipmes;

import java.util.ArrayList;

public class Join {
    DependencyGraph temporalRelation;
    PatternGraph spatialRelation;

    public Join(DependencyGraph temporalRelation, PatternGraph spatialRelation) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
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
     * Detect whether any edge in subTCQ appear in entry
     * 
     * @param subTCQ TC sub-query
     * @param entry  the matched entry
     * @return true if they share a pattern edge
     */
    private boolean overlap(ArrayList<DataEdge> subTCQ, ArrayList<DataEdge> entry) {
        for (DataEdge i : subTCQ) {
            for (DataEdge j : entry) {
                if (i.matched.getId().equals(j.matched.getId()))
                    return true;
            }
        }
        return false;
    }

    /**
     * Join the match result of TC sub-queries
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
    public ArrayList<ArrayList<Integer>> joinMatchResult() {
        ArrayList<ArrayList<DataEdge>> matchResult = new ArrayList<ArrayList<DataEdge>>();
        ArrayList<ArrayList<DataEdge>> expansionTable = new ArrayList<ArrayList<DataEdge>>();
        ArrayList<ArrayList<Integer>> answer = new ArrayList<ArrayList<Integer>>();
        boolean fit = true;
        int numEdges = this.spatialRelation.getEdges().size();
        for (ArrayList<DataEdge> subTCQ : matchResult) {
            for (ArrayList<DataEdge> entry : expansionTable) {
                fit = true;
                if (overlap(subTCQ, entry))
                    continue;
                for (DataEdge edgeInMatchResult : subTCQ) {
                    for (DataEdge edgeInTable : entry) {
                        if (!(checkRelation(edgeInMatchResult, edgeInTable)
                                && checkTime(edgeInMatchResult, edgeInTable))) {
                            fit = false;
                            break;
                        }
                    }
                    if (!fit)
                        break;
                }
                if (fit) {
                    entry.addAll(subTCQ);
                    if (entry.size() == numEdges) {
                        ArrayList<Integer> tmp = new ArrayList<Integer>();
                        for (DataEdge edge : entry) {
                            tmp.add(edge.getDataId());
                        }
                        answer.add(tmp);
                    }
                }
            }
        }
        return answer;
    }
}
