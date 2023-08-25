package org.ipmes.join;

import org.ipmes.decomposition.TCQuery;
import org.ipmes.decomposition.TCQueryRelation;
import org.ipmes.match.FullMatch;
import org.ipmes.match.MatchEdge;
import org.ipmes.match.MatchResult;
import org.ipmes.pattern.TemporalRelation;
import org.ipmes.pattern.PatternGraph;

import java.util.*;

/**
 * The organized table approach to join partial matches.
 */
public class PriorityJoin implements Join {
    TemporalRelation temporalRelation;
    PatternGraph spatialRelation;
    // store the match result of the whole pattern
    HashSet<FullMatch> answer;
    // table for joining result
    PriorityQueue<MatchResult>[] partialMatchResult;
    // store the realtionships of sub TC Queries
    PriorityGenRel relationGenerator;
    ArrayList<TCQueryRelation>[] TCQRelation;
    long windowSize;
    int curPoolSize;
    Integer[] usageCount;

    // constructor
    public PriorityJoin(TemporalRelation temporalRelation, PatternGraph spatialRelation, long windowSize,
            ArrayList<TCQuery> subTCQueries) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
        this.answer = new HashSet<>();
        this.relationGenerator = new PriorityGenRel(temporalRelation, spatialRelation, subTCQueries);
        this.TCQRelation = relationGenerator.getRelation();
        this.windowSize = windowSize;
        this.partialMatchResult = (PriorityQueue<MatchResult>[]) new PriorityQueue[2 * TCQRelation.length - 1];
        for (int i = 0; i < TCQRelation.length; i++) {
            this.partialMatchResult[i] = new PriorityQueue<>(Comparator.comparingLong(MatchResult::getEarliestTime));
        }
        this.curPoolSize = 0;
        this.usageCount = new Integer[subTCQueries.size()];
        for (int i = 0; i < usageCount.length; i++) {
            this.usageCount[i] = 0;
        }
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
     * @param edgeInEntry
     * @return true if spatial relation between dataEdge and patternEdge is the
     *         same, otherwise, false.
     */
    private boolean checkSpatialRelation(MatchEdge edgeInMatchResult, MatchEdge edgeInEntry) {
        Long[][] arr = {
                edgeInMatchResult.getMatched().getEndpoints(),
                edgeInEntry.getMatched().getEndpoints(),
                edgeInMatchResult.getEndpoints(),
                edgeInEntry.getEndpoints()
        };
        return spatialRelationType(arr[0], arr[1]) == spatialRelationType(arr[2], arr[3]);
    }

    /**
     * check edge temporal relation
     * 
     * @param edgeInMatchResult
     * @param edgeInEntry
     * @return true if temporal relation between dataEdge and patternEdge is the
     *         same, otherwise, false.
     */
    private boolean checkTemporalRelation(MatchEdge edgeInMatchResult, MatchEdge edgeInEntry) {
        return (this.temporalRelation.getParents(edgeInMatchResult.matchId())
                .contains(edgeInEntry.matchId()) && edgeInMatchResult.getTimestamp() >= edgeInEntry.getTimestamp())
                ||
                (this.temporalRelation.getChildren(edgeInMatchResult.matchId())
                        .contains(edgeInEntry.matchId())
                        && edgeInMatchResult.getTimestamp() <= edgeInEntry.getTimestamp())
                ||
                (!this.temporalRelation.getChildren(edgeInMatchResult.matchId())
                        .contains(edgeInEntry.matchId())
                        && !this.temporalRelation.getParents(edgeInMatchResult.matchId())
                                .contains(edgeInEntry.matchId()));
    }

    /**
     * check whether the result and the entry fit the relations.
     * 
     * @param result
     * @param entry
     * @param bufferId
     * @return true if fit all the relations.
     */
    boolean checkRelations(MatchResult result, MatchResult entry, int bufferId) {
        for (TCQueryRelation relation : this.TCQRelation[bufferId]) {
            if (!(checkSpatialRelation(result.get(relation.idOfResult),
                    entry.get(relation.idOfEntry))
                    && checkTemporalRelation(result.get(relation.idOfResult),
                            entry.get(relation.idOfEntry)))) {
                return false;
            }
        }
        return true;
    }

    /**
     * join new entries with sibling buffer
     * 
     * @param newEntries
     * @param bufferId
     * @return the results of join with sibling buffer
     */
    ArrayList<MatchResult> joinWithSibling(ArrayList<MatchResult> newEntries, int bufferId) {
        int siblingId = getSibling(bufferId);
        ArrayList<MatchResult> ret = new ArrayList<>();
        for (MatchResult result : newEntries) {
            for (MatchResult entry : this.partialMatchResult[siblingId]) {
                if (checkRelations(result, entry, bufferId))
                    ret.add(entry.merge(result));
            }
        }
        return ret;
    }

    /**
     * clean up expired partial matches
     * 
     * @param latestTime current time
     * @param bufferId   the buffer we want to clean up
     */
    void clearExpired(long latestTime, int bufferId) {
        while (!this.partialMatchResult[bufferId].isEmpty() &&
                latestTime - this.windowSize > this.partialMatchResult[bufferId].peek().getEarliestTime()) {
            this.partialMatchResult[bufferId].poll();
            this.curPoolSize -= 1;
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
        this.usageCount[tcQueryId] += 1;
        long latestTime = result.getLatestTime();
        // join
        int bufferId = toBufferIdx(tcQueryId);
        ArrayList<MatchResult> newEntries = new ArrayList<>();
        newEntries.add(result);
        while (!newEntries.isEmpty()) {
            if (bufferId == TCQRelation.length - 1) {
                for (MatchResult res : newEntries)
                    answer.add(res.toFullMatch());
                break;
            }
            this.partialMatchResult[bufferId].addAll(newEntries);
            this.curPoolSize += newEntries.size();
            clearExpired(latestTime, getSibling(bufferId));
            newEntries = joinWithSibling(newEntries, bufferId);
            bufferId = getParent(bufferId);
        }
    }

    /**
     * extract full match result.
     */
    public Collection<FullMatch> extractAnswer() {
        Collection<FullMatch> res = this.answer;
        this.answer = new HashSet<>();
        return res;
    }

    public int getPoolSize() {
        return this.curPoolSize;
    }

    public Integer[] getUsageCount() {
        return this.usageCount;
    }
}
