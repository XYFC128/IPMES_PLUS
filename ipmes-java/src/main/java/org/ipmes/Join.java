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

    private boolean checkRelation(DataEdge edgeInMatchResult, DataEdge edgeInTable) {
        int arr[][] = {
                { edgeInMatchResult.matched.getStartId(), edgeInMatchResult.matched.getEndId() },
                { edgeInTable.matched.getStartId(), edgeInTable.matched.getEndId() },
                { edgeInMatchResult.getStartId(), edgeInMatchResult.getEndId() },
                { edgeInTable.getStartId(), edgeInTable.getEndId() }
        };
        return (relationType(arr[0], arr[1]) == relationType(arr[2], arr[3]));
    }

    private boolean checkTime(DataEdge edgeInMatchResult, DataEdge edgeInTable) {
        return (this.temporalRelation.getParents(edgeInMatchResult.matched.getId())
                .contains(edgeInTable.matched.getId())
                && edgeInMatchResult.timestamp > edgeInTable.timestamp)
                ||
                (this.temporalRelation.getChildren(edgeInMatchResult.matched.getId())
                        .contains(edgeInTable.matched.getId())
                        && edgeInMatchResult.timestamp < edgeInTable.timestamp)
                ||
                (!this.temporalRelation.getParents(edgeInMatchResult.matched.getId())
                        .contains(edgeInTable.matched.getId())
                        && !this.temporalRelation.getChildren(edgeInMatchResult.matched.getId())
                                .contains(edgeInTable.matched.getId()));
    }

    // detect whether any edge in subTCQ appear in entry
    private boolean overlap(ArrayList<DataEdge> subTCQ, ArrayList<DataEdge> entry) {
        for (DataEdge i : subTCQ) {
            for (DataEdge j : entry) {
                if (i.matched.getId() == j.matched.getId())
                    return true;
            }
        }
        return false;
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
    public ArrayList<ArrayList<Integer>> joinMatchResult() {
        ArrayList<ArrayList<DataEdge>> matchResult = new ArrayList<ArrayList<DataEdge>>();
        ArrayList<ArrayList<DataEdge>> expansionTable = new ArrayList<ArrayList<DataEdge>>();
        ArrayList<ArrayList<Integer>> answer = new ArrayList<ArrayList<Integer>>();
        boolean fit = true;
        int totalSize = this.spatialRelation.getEdges().size();
        for (ArrayList<DataEdge> subTCQ : matchResult) {
            for (ArrayList<DataEdge> entry : expansionTable) {
                fit = true;
                if (overlap(subTCQ, entry))
                    continue;
                for (DataEdge edgeInMatchResult : subTCQ) {
                    for (DataEdge edgeInTable : entry) {
                        if (!checkRelation(edgeInMatchResult, edgeInTable)
                                || !checkTime(edgeInMatchResult, edgeInTable)) {
                            fit = false;
                            break;
                        }
                    }
                    if (!fit)
                        break;
                }
                if (fit) {
                    entry.addAll(subTCQ);
                    if (entry.size() == totalSize) {
                        ArrayList<Integer> tmp = new ArrayList<Integer>();
                        for (DataEdge edge : entry) {
                            tmp.add(edge.getDataId());
                        }
                        answer.add(tmp);
                    }
                }
            }
        }
        return answer;
    }
}
