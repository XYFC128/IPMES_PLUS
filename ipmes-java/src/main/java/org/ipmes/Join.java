package org.ipmes;

import org.ipmes.decomposition.TCQueryRelation;
import org.ipmes.match.MatchEdge;
import org.ipmes.match.MatchResult;
import org.ipmes.pattern.DependencyGraph;
import org.ipmes.pattern.PatternGraph;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.SortedMap;
import java.util.TreeMap;

public class Join {
    DependencyGraph temporalRelation;
    PatternGraph spatialRelation;
    // store the match result of the whole pattern
    ArrayList<MatchResult> answer;
    // table for joining result
    HashSet<MatchResult> expansionTable;
    // store the realtionships of sub TC Queries
    ArrayList<TCQueryRelation>[] TCQRelation;
    // use SortedMap<TimeStamp, entry> to maintain window
    TreeMap<Integer, MatchResult> mapForWindow;
    int windowSize;

    public Join(DependencyGraph temporalRelation, PatternGraph spatialRelation,
            ArrayList<TCQueryRelation>[] TCQRelation, int windowSize) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
        this.answer = new ArrayList<MatchResult>();
        this.expansionTable = new HashSet<MatchResult>();
        this.TCQRelation = TCQRelation;
        this.mapForWindow = new TreeMap<Integer, MatchResult>();
        this.windowSize = windowSize;
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
                edgeInMatchResult.getMatched().getEndpoints(),
                edgeInTable.getMatched().getEndpoints(),
                edgeInMatchResult.getEndpoints(),
                edgeInTable.getEndpoints()
        };
        return relationType(arr[0], arr[1]) == relationType(arr[2], arr[3]);
    }

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
        while (!this.mapForWindow.isEmpty()
                && result.getLatestTime() - this.windowSize > this.mapForWindow.firstKey()) {
            MatchResult nextToRemove = this.mapForWindow.firstEntry().getValue();
            while (nextToRemove != null) {
                MatchResult tmp = nextToRemove;
                nextToRemove = nextToRemove.getNext();
                this.expansionTable.remove(tmp);
            }
            // remove from FIFO
            this.mapForWindow.pollFirstEntry();
        }
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
            else {
                expansionTable.add(newEntry);
                if (this.mapForWindow.containsKey(newEntry.getEarliestTime()))
                    newEntry.setNext(this.mapForWindow.get(newEntry.getEarliestTime()));
                this.mapForWindow.put(newEntry.getEarliestTime(), newEntry);
            }
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
