package org.ipmes;

import org.ipmes.decomposition.TCQueryRelation;
import org.ipmes.match.MatchEdge;
import org.ipmes.match.MatchResult;
import org.ipmes.pattern.DependencyGraph;
import org.ipmes.pattern.PatternGraph;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashSet;
import java.util.PriorityQueue;

public class Join {
    DependencyGraph temporalRelation;
    PatternGraph spatialRelation;
    // store the match result of the whole pattern
    ArrayList<MatchResult> answer;
    HashSet<MatchResult> expansionTable;
    // table for joining result
    PriorityQueue<MatchResult>[] partialMatchResult;
    // store the realtionships of sub TC Queries
    ArrayList<TCQueryRelation>[] TCQRelation;
    long windowSize;

    // constructor
    public Join(DependencyGraph temporalRelation, PatternGraph spatialRelation,
            ArrayList<TCQueryRelation>[] TCQRelation, long windowSize) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
        this.answer = new ArrayList<MatchResult>();
        this.TCQRelation = TCQRelation;
        this.expansionTable = new HashSet<MatchResult>();
        this.windowSize = windowSize;
        this.partialMatchResult = (PriorityQueue<MatchResult>[]) new PriorityQueue[2 * TCQRelation.length - 1];
        for (int i = 0; i < 2 * TCQRelation.length - 1; i++) {
            this.partialMatchResult[i] = new PriorityQueue<MatchResult>(new Comparator<MatchResult>() {
                public int compare(MatchResult result1, MatchResult result2) {
                    if (result1.getEarliestTime() > result2.getEarliestTime())
                        return 1;
                    else if (result1.getEarliestTime() == result2.getEarliestTime())
                        return 0;
                    return -1;
                }
            });
        }
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
    private byte spatialRelationType(Long[] n, Long[] m) {
        byte ret = 0;
        for (long i : n) {
            for (long j : m) {
                if (i == j)
                    ret |= 1;
                ret <<= 1;
            }
        }
        return ret;
    }

    /**
     * check edge spatial relation
     * 
     * @param edgeInMatchResult
     * @param edgeInTable
     * @return true if spatial relation between dataEdge and patternEdge is the
     *         same, otherwise, false.
     */

    private boolean checkSpatialRelation(MatchEdge edgeInMatchResult, MatchEdge edgeInTable) {
        Long[][] arr = {
                edgeInMatchResult.getMatched().getEndpoints(),
                edgeInTable.getMatched().getEndpoints(),
                edgeInMatchResult.getEndpoints(),
                edgeInTable.getEndpoints()
        };
        return spatialRelationType(arr[0], arr[1]) == spatialRelationType(arr[2], arr[3]);
    }

    /**
     * check edge temporal relation
     * 
     * @param edgeInMatchResult
     * @param edgeInTable
     * @return true if temporal relation between dataEdge and patternEdge is the
     *         same, otherwise, false.
     */

    private boolean checkTime(MatchEdge edgeInMatchResult, MatchEdge edgeInTable) {
        return (this.temporalRelation.getParents(edgeInMatchResult.matchId())
                .contains(edgeInTable.matchId()) && edgeInMatchResult.getTimestamp() >= edgeInTable.getTimestamp())
                ||
                (this.temporalRelation.getChildren(edgeInMatchResult.matchId())
                        .contains(edgeInTable.matchId())
                        && edgeInMatchResult.getTimestamp() <= edgeInTable.getTimestamp())
                ||
                (!this.temporalRelation.getChildren(edgeInMatchResult.matchId())
                        .contains(edgeInTable.matchId())
                        && !this.temporalRelation.getParents(edgeInMatchResult.matchId())
                                .contains(edgeInTable.matchId()));
    }

    // private boolean checkRelation(MatchResult result, MatchResult entry,
    // TCQueryRelation rel) {
    // boolean ret = true;
    // if ((rel.relationType & 4) == 4) {
    // ret &= checkSpatialRelation(result.get(rel.idOfResult),
    // entry.get(rel.idOfEntry));
    // }
    // if ((rel.relationType & 2) == 2) {
    // ret &= this.temporalRelation.getParents(result.get(rel.idOfResult).matchId())
    // .contains(entry.get(rel.idOfEntry).matchId());
    // }
    // if ((rel.relationType & 1) == 1) {
    // ret &=
    // this.temporalRelation.getChildren(result.get(rel.idOfResult).matchId())
    // .contains(entry.get(rel.idOfEntry).matchId());
    // }
    // }

    private void joinTwoTable(PriorityQueue<MatchResult> pqWithoutRel, PriorityQueue<MatchResult> pqWithRel, int id) {
        boolean fit = true;
        for (MatchResult result : pqWithRel) {
            for (MatchResult entry : pqWithoutRel) {
                fit = true;
                for (TCQueryRelation relationship : this.TCQRelation[(id + 1) / 2]) {
                    if (entry.containsPattern(relationship.idOfEntry)) {
                        if (!(checkSpatialRelation(result.get(relationship.idOfResult),
                                entry.get(relationship.idOfEntry))
                                && checkTime(result.get(relationship.idOfResult),
                                        entry.get(relationship.idOfEntry)))) {
                            fit = false;
                            break;
                        }
                    }
                }
                if (fit) {
                    addMatchResult(result.merge(entry), id + 1);
                }
            }
        }
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
    public void addMatchResult(MatchResult result, Integer tcQueryId) {
        // check uniqueness of the MatchResult
        if (this.expansionTable.contains(result))
            return;
        this.expansionTable.add(result);
        // join

        // cleanOutOfDate(result.getEarliestTime(), tcQueryId);
        if (tcQueryId == 2 * this.TCQRelation.length - 2) {
            this.answer.add(result);
            return;
        }
        this.partialMatchResult[tcQueryId].add(result);
        if (tcQueryId == 0)
            return;
        PriorityQueue<MatchResult> pqForNew = new PriorityQueue<MatchResult>(new Comparator<MatchResult>() {
            public int compare(MatchResult result1, MatchResult result2) {
                if (result1.getEarliestTime() > result2.getEarliestTime())
                    return 1;
                else if (result1.getEarliestTime() == result2.getEarliestTime())
                    return 0;
                return -1;
            }
        });
        pqForNew.add(result);

        if (tcQueryId % 2 == 0)
            joinTwoTable(pqForNew, this.partialMatchResult[tcQueryId + 1], tcQueryId +
                    1);
        else
            joinTwoTable(this.partialMatchResult[tcQueryId - 1], pqForNew, tcQueryId);
        return;
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
