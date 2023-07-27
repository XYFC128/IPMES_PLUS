package org.ipmes;

import java.util.ArrayList;

public class Join {

    private boolean checkRelation(DataEdge candidate, DataEdge AlreadyIn) {

        return true;
    }

    private boolean checkTime(DataEdge candidate, DataEdge AlreadyIn) {

        return true;
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
        ArrayList<ArrayList<DataEdge>> MatchResult = new ArrayList<ArrayList<DataEdge>>();
        ArrayList<ArrayList<DataEdge>> ExpansionTable = new ArrayList<ArrayList<DataEdge>>();
        boolean fit = true;
        for (ArrayList<DataEdge> subTCQ : MatchResult) {
            for (ArrayList<DataEdge> entry : ExpansionTable) {
                fit = true;
                for (DataEdge candidate : subTCQ) {
                    for (DataEdge AlreadyIn : entry) {
                        if (!checkRelation(candidate, AlreadyIn) || !checkTime(candidate, AlreadyIn)) {
                            fit = false;
                            break;
                        }
                    }
                    if (!fit)
                        break;
                }
                // if(fit)
                // put subTCQ in this entry;
            }
        }
    }
}
