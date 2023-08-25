package org.ipmes.match;

import java.util.Arrays;

/**
 * Holds the full match of a pattern.
 */
public class FullMatch {
    long[] matchData;

    public FullMatch(int size) {
        this.matchData = new long[size];
    }

    public void set(int patternId, long dataId) {
        matchData[patternId] = dataId;
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
