package org.ipmes;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.Map;

public class Join {
    DependencyGraph temporalRelation;
    PatternGraph spatialRelation;
    // store the match result of the whole pattern
    ArrayList<MatchResult> answer;
    // table for joining result
    HashSet<MatchResult> expansionTable;
    // store the realtionships of sub TC Queries
    ArrayList<TCQueryRelation>[] TCQRelation;

    public Join(DependencyGraph temporalRelation, PatternGraph spatialRelation,
            ArrayList<TCQueryRelation>[] TCQRelation) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
        this.answer = new ArrayList<MatchResult>();
        this.expansionTable = new HashSet<MatchResult>();
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
                .contains(edgeInTable.matched.getId()) && edgeInMatchResult.timestamp >= edgeInTable.timestamp)
                ||
                (this.temporalRelation.getChildren(edgeInMatchResult.matched.getId())
                        .contains(edgeInTable.matched.getId()) && edgeInMatchResult.timestamp <= edgeInTable.timestamp)
                ||
                (!this.temporalRelation.getChildren(edgeInMatchResult.matched.getId())
                        .contains(edgeInTable.matched.getId())
                        && !this.temporalRelation.getParents(edgeInMatchResult.matched.getId())
                                .contains(edgeInTable.matched.getId()));
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
    public void addMatchResult(MatchResult result, Integer tcQueryId) {
        if (this.expansionTable.contains(result))
            return;
        boolean fit = true;
        ArrayList<MatchResult> buffer = new ArrayList<>();
        // join
        for (MatchResult entry : this.expansionTable) {
            if (entry.hasShareEdge(result))
                continue;
            fit = true;
            for (TCQueryRelation relationship : this.TCQRelation[tcQueryId]) {
                if (entry.containsPattern(relationship.idOfEntry)) {
                    if (!(checkRelation(result.get(relationship.idOfResult), entry.get(relationship.idOfEntry))
                            && checkTime(result.get(relationship.idOfResult), entry.get(relationship.idOfEntry)))) {
                        fit = false;
                        break;
                    }
                }
            }
            if (fit) {
                buffer.add(result.merge(entry));
            }
        }
        // insert
        buffer.add(result);
        int ansSize = this.spatialRelation.numEdges();
        for (MatchResult newEntry : buffer) {
            if (newEntry.size() == ansSize)
                answer.add(newEntry);
            else
                expansionTable.add(newEntry);
        }
    }

    public ArrayList<ArrayList<MatchEdge>> extractAnswer() {
        ArrayList<ArrayList<MatchEdge>> ret = new ArrayList<ArrayList<MatchEdge>>();
        for (MatchResult result : this.answer) {
            ret.add(new ArrayList<>(result.matchEdges()));
        }
        this.answer.clear();
        return ret;
    }
}
