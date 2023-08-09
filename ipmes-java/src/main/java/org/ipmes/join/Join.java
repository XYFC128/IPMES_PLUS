package org.ipmes.join;

import java.util.ArrayList;

import org.ipmes.match.MatchEdge;
import org.ipmes.match.MatchResult;

public interface Join {
    public void addMatchResult(MatchResult result, Integer tcQueryId);

    public ArrayList<ArrayList<MatchEdge>> extractAnswer();
}
