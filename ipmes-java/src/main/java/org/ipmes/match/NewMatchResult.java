package org.ipmes.match;

import java.util.*;

public class NewMatchResult {
    public static int MAX_NUM_EDGES;
    public static int MAX_NUM_NODES;
    int hash;
    long[] edgeIdMap;
    int numEdges;
    long[] nodeIdMap;
    int numNodes;
    long earliestTime;
    long latestTime;

    public NewMatchResult() {
        this.hash = 0;
        this.earliestTime = Long.MAX_VALUE;
        this.latestTime = Long.MIN_VALUE;
    }

    /**
     * Calculate a^n mod p
     */
    static long pow(long a, long n, long p) {
        long res = 1;
        long base = a;
        while (n > 0) {
            if ((n & 1) == 1)
                res = (res * base) % p;
            base = (base * base) % p;
            n >>= 1;
        }
        return res;
    }

    public void addMatchEdge(MatchEdge m) {
        Integer matchId = m.matched.getId();
        this.hash += (int)(m.getDataId() * pow(7, matchId, Integer.MAX_VALUE) % Integer.MAX_VALUE);
        this.earliestTime = Math.min(this.earliestTime, m.timestamp);
        this.latestTime = Math.max(this.latestTime, m.timestamp);
    }


    public long getEarliestTime() {
        return this.earliestTime;
    }

    public long getLatestTime() {
        return this.latestTime;
    }

    @Override
    public int hashCode() {
        return this.hash;
    }

    @Override
    public boolean equals(Object obj) {
        if (obj == this)
            return true;
        if (!(obj instanceof NewMatchResult))
            return false;
        NewMatchResult other = (NewMatchResult) obj;
        return true;
    }
}
