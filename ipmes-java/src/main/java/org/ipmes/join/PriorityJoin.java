package org.ipmes.join;

import org.ipmes.decomposition.TCQueryRelation;
import org.ipmes.match.MatchEdge;
import org.ipmes.match.MatchResult;
import org.ipmes.pattern.DependencyGraph;
import org.ipmes.pattern.PatternGraph;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashSet;
import java.util.PriorityQueue;

public class PriorityJoin implements Join {
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
    public PriorityJoin(DependencyGraph temporalRelation, PatternGraph spatialRelation,
            ArrayList<TCQueryRelation>[] TCQRelation, long windowSize) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
        this.answer = new ArrayList<MatchResult>();
        this.TCQRelation = TCQRelation;
        this.expansionTable = new HashSet<MatchResult>();
        this.windowSize = windowSize;
        this.partialMatchResult = (PriorityQueue<MatchResult>[]) new PriorityQueue[2 * TCQRelation.length - 1];
        for (int i = 0; i < 2 * TCQRelation.length - 1; i++) {
            this.partialMatchResult[i] = new PriorityQueue<>(Comparator.comparingLong(MatchResult::getEarliestTime));
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

    private boolean checkTemporalRelation(MatchEdge edgeInMatchResult, MatchEdge edgeInTable) {
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

    void checkAndMerge(MatchResult left, MatchResult right, int haveRelId, ArrayList<MatchResult> ret) {
        boolean fit = true;
        for (TCQueryRelation relation : this.TCQRelation[haveRelId]) {
            if (left.containsPattern(relation.idOfEntry)) {
                if (!(checkSpatialRelation(right.get(relation.idOfResult),
                        left.get(relation.idOfEntry))
                        && checkTemporalRelation(right.get(relation.idOfResult),
                                left.get(relation.idOfEntry)))) {
                    fit = false;
                    break;
                }
            }
        }
        if (fit)
            ret.add(left.merge(right));
    }

    // Collection<MatchResult> joinTwoTable(Collection<MatchResult> newResults, int
    ArrayList<MatchResult> process(ArrayList<MatchResult> toProcess, int bufferId) {
        int siblingId = getSibling(bufferId), haveRelId;
        ArrayList<MatchResult> ret = new ArrayList<MatchResult>();
        for (MatchResult result : toProcess) {
            this.partialMatchResult[bufferId].add(result);
            if (bufferId == 0)
                continue;
            if (bufferId == 2 * TCQRelation.length - 2) {
                this.answer.add(result);
                continue;
            }
            if (bufferId % 2 == 0) {
                for (MatchResult right : this.partialMatchResult[siblingId]) {
                    haveRelId = toTCQueryId(siblingId);
                    checkAndMerge(result, right, haveRelId, ret);
                }
            } else {
                for (MatchResult left : this.partialMatchResult[siblingId]) {
                    haveRelId = toTCQueryId(bufferId);
                    checkAndMerge(left, result, haveRelId, ret);
                }
            }
        }
        return ret;
    }

    int toBufferIdx(int tcQueryId) {
        if (tcQueryId == 0)
            return 0;
        return tcQueryId * 2 - 1;
    }

    int toTCQueryId(int bufferId) {
        return (bufferId + 1) / 2;
    }

    int getSibling(int bufferId) {
        if ((bufferId & 1) == 1)
            return bufferId - 1;
        return bufferId + 1;
    }

    int getParent(int bufferId) {
        if ((bufferId & 1) == 1)
            return bufferId + 1;
        return bufferId + 2;
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
        int bufferId = toBufferIdx(tcQueryId);
        ArrayList<MatchResult> toProcess = new ArrayList<MatchResult>();
        toProcess.add(result);
        while (!toProcess.isEmpty()) {
            toProcess = process(toProcess, bufferId);
            bufferId = getParent(bufferId);
        }
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
