package org.ipmes;

import java.util.ArrayList;

public class Join {

    DependencyGraph temporalRelation;
    PatternGraph spatialRelation;

    public Join(DependencyGraph temporalRelation, PatternGraph spatialRelation) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
    }

    private Integer relationType(int n[], int m[]) {
        int ret = 0;
        for (int i : n) {
            for (int j : m) {
                if (i == j)
                    ret += 1;
                ret *= 2;
            }
        }
        return ret;
    }

    private boolean checkRelation(DataEdge candidate, DataEdge alreadyIn) {
        int arr[][] = {
                { candidate.matched.getStartId(), candidate.matched.getEndId() },
                { alreadyIn.matched.getStartId(), alreadyIn.matched.getEndId() },
                { candidate.getStartId(), candidate.getEndId() },
                { alreadyIn.getStartId(), alreadyIn.getEndId() }
        };
        return (relationType(arr[0], arr[1]) == relationType(arr[2], arr[3]));
    }

    private boolean checkTime(DataEdge candidate, DataEdge alreadyIn) {
        return (this.temporalRelation.getParents(candidate.matched.getId()).contains(alreadyIn.matched.getId())
                && candidate.timestamp > alreadyIn.timestamp)
                ||
                (this.temporalRelation.getChildren(candidate.matched.getId()).contains(alreadyIn.matched.getId())
                        && candidate.timestamp < alreadyIn.timestamp)
                ||
                (!this.temporalRelation.getParents(candidate.matched.getId()).contains(alreadyIn.matched.getId())
                        && !this.temporalRelation.getChildren(candidate.matched.getId())
                                .contains(alreadyIn.matched.getId()));
    }

    /*
     * ///////////////////////////////////////////////
     * join the result of TC subquery matching
     *////////////////////////////////////////////////

    // output of Siddhi App would be ArrayList<MatchEdge>
    //
    // DataEdge{
    // dataId: edge id in the input data
    // timestamp: timestamp in the input data
    // startId: node id in the input data
    // endId: node id in the input data
    // matched: 0 based id we given to the edge
    // }
    public void joinMatchResult() {
        ArrayList<ArrayList<DataEdge>> matchResult = new ArrayList<ArrayList<DataEdge>>();
        ArrayList<ArrayList<DataEdge>> expansionTable = new ArrayList<ArrayList<DataEdge>>();
        boolean fit = true;
        for (ArrayList<DataEdge> subTCQ : matchResult) {
            for (ArrayList<DataEdge> entry : expansionTable) {
                fit = true;
                for (DataEdge candidate : subTCQ) {
                    for (DataEdge alreadyIn : entry) {
                        if (!checkRelation(candidate, alreadyIn) || !checkTime(candidate, alreadyIn)) {
                            fit = false;
                            break;
                        }
                    }
                    if (!fit)
                        break;
                }
                if (fit)
                    entry.addAll(subTCQ);
            }
        }
    }
}
