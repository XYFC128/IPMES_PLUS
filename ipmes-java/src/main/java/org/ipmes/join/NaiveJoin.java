package org.ipmes.join;

import org.ipmes.decomposition.TCQueryRelation;
import org.ipmes.match.MatchEdge;
import org.ipmes.match.MatchResult;
import org.ipmes.pattern.DependencyGraph;
import org.ipmes.pattern.PatternGraph;
import org.ipmes.decomposition.TCQuery;
import org.ipmes.join.NaiveGenRel;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.TreeMap;

public class NaiveJoin implements Join {

    DependencyGraph temporalRelation;
    PatternGraph spatialRelation;
    // store the match result of the whole pattern
    ArrayList<MatchResult> answer;
    // table for joining result
    HashSet<MatchResult> expansionTable;
    // use SortedMap<TimeStamp, entry> to maintain window
    TreeMap<Long, MatchResult> mapForWindow;
    long windowSize;
    // all the new entry will be stored in bufferForPartialMatch,
    // and add to table at the end of addMatchResult
    ArrayList<MatchResult> bufferForPartialMatch;
    // store the realtionships of sub TC Queries
    NaiveGenRel relationGenerator;
    ArrayList<TCQueryRelation>[] TCQRelation;

    // constructor
    public NaiveJoin(DependencyGraph temporalRelation, PatternGraph spatialRelation, long windowSize,
            ArrayList<TCQuery> subTCQueries) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
        this.answer = new ArrayList<MatchResult>();
        this.expansionTable = new HashSet<MatchResult>();
        this.relationGenerator = new NaiveGenRel(temporalRelation, spatialRelation, subTCQueries);
        this.TCQRelation = relationGenerator.getRelation();
        this.mapForWindow = new TreeMap<Long, MatchResult>();
        this.windowSize = windowSize;
        this.bufferForPartialMatch = new ArrayList<MatchResult>();
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

    private boolean checkTime(MatchEdge edgeInMatchResult, MatchEdge edgeInEntry) {
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
     * use Timeout Window to clean up out-of-date entry.
     * <p>
     * we use linked list to link all the entry with the same earliestTime,
     * so when one is out-of-date, we can easily remove all the entry in the linked
     * list.
     * </p>
     * 
     * @param time the timestamp we are processing
     */
    private void cleanExpansionTable(long time) {
        while (!this.mapForWindow.isEmpty()) {
            if ((time - this.windowSize) < this.mapForWindow.firstKey())
                break;
            MatchResult nextToRemove = this.mapForWindow.firstEntry().getValue();
            while (nextToRemove != null) {
                MatchResult tmp = nextToRemove;
                nextToRemove = nextToRemove.getNext();
                this.expansionTable.remove(tmp);
            }
            // remove from FIFO
            this.mapForWindow.pollFirstEntry();
        }
        return;
    }

    void joinMatchResult(MatchResult result, int tcQueryId) {
        boolean fit = true;
        for (MatchResult entry : this.expansionTable) {
            // check whether entry and result overlap
            if (entry.hasShareEdge(result))
                continue;
            fit = true;
            // if any pair of edges in entry and result break the rules,
            // change fit to false and break(the result doesn't fit in the entry)
            for (TCQueryRelation relationship : this.TCQRelation[tcQueryId]) {
                if (entry.containsPattern(relationship.idOfEntry)) {
                    if (!(checkSpatialRelation(result.get(relationship.idOfResult), entry.get(relationship.idOfEntry))
                            && checkTime(result.get(relationship.idOfResult), entry.get(relationship.idOfEntry)))) {
                        fit = false;
                        break;
                    }
                }
            }
            if (fit) {
                this.bufferForPartialMatch.add(result.merge(entry));
            }
        }
    }

    private void insertToTable() {
        int ansSize = this.spatialRelation.numEdges();
        for (MatchResult entry : this.bufferForPartialMatch) {
            if (entry.size() == ansSize)
                this.answer.add(entry);
            else {
                this.expansionTable.add(entry);
                if (this.mapForWindow.containsKey(entry.getEarliestTime()))
                    entry.setNext(this.mapForWindow.get(entry.getEarliestTime()));
                this.mapForWindow.put(entry.getEarliestTime(), entry);
            }
        }
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
        // use Timeout Window to clean useless entry
        cleanExpansionTable(result.getLatestTime());
        // join
        joinMatchResult(result, tcQueryId);
        this.bufferForPartialMatch.add(result);
        // add the entry in bufferForPartialMatch to the expansionTable
        insertToTable();
        this.bufferForPartialMatch.clear();
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
