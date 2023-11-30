package org.ipmes.match;

import java.util.Arrays;

/**
 * Holds the full match of a pattern.
 */
public class FullMatch {
    long[] matchData;
    long startTime;
    long endTime;

    public FullMatch(int size) {
        this.matchData = new long[size];
        this.startTime = Long.MAX_VALUE;
        this.endTime = Long.MIN_VALUE;
    }

    public void set(int patternId, MatchEdge matchEdge) {
        matchData[patternId] = matchEdge.getDataId();
        long timestamp = matchEdge.getTimestamp();
        this.startTime = Math.min(timestamp, this.startTime);
        this.endTime = Math.max(timestamp, this.endTime);
    }

    public long[] getMatchData() {
        return matchData;
    }

    public long getStartTime() {
        return startTime;
    }

    public long getEndTime() {
        return endTime;
    }

    @Override
    public String toString() {
        return Arrays.toString(this.matchData);
    }

    @Override
    public int hashCode() {
        return Arrays.hashCode(this.matchData);
    }

    @Override
    public boolean equals(Object obj) {
        if (obj == this)
            return true;
        if (!(obj instanceof FullMatch))
            return false;
        FullMatch other = (FullMatch) obj;
        return Arrays.equals(this.matchData, other.matchData);
    }
}
