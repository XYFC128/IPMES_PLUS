package org.ipmes.join;

import java.util.ArrayList;
import java.util.Collection;

import org.ipmes.match.MatchEdge;
import org.ipmes.match.MatchResult;

public interface Join {
    public void addMatchResult(MatchResult result, Integer tcQueryId);

    public Collection<long[]> extractAnswer();
}
