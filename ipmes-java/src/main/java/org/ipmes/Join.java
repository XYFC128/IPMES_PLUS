package org.ipmes;

import java.util.ArrayList;

public class Join {

    /*
     * ///////////////////////////////////////////////
     * join the result of TC subquery matching
     *////////////////////////////////////////////////

    // output of Siddhi App would be ArrayList<MatchEdge>
    //
    // MatchEdge{
    // eventId: edge id in the input data
    // timestamp: timestamp in the input data
    // startId: node id in the input data
    // endId: node id in the input data
    // patternId: 0 based id we given to the edge
    // }

    ArrayList<ArrayList<DataEdge>> MatchResult = new ArrayList<ArrayList<DataEdge>>();
    ArrayList<ArrayList<DataEdge>> ExpansionTable = new ArrayList<ArrayList<DataEdge>>();for(
    ArrayList<DataEdge> subTCQ:MatchResult){
            for(ArrayList<DataEdge> entry : ExpansionTable){
                fit = true;
                for(DataEdge candidate : subTCQ){
                    for(DataEdge AlreadyIn : entry){
                        if(!checkRelation(candidate, AlreadyIn) || !checkTime(cnadidate, AlreadyIn)){
                            fit = false;
                            break;
                        }
                    }
                    if(!fit)
                        break;
                }
                if(fit)
                    // put subTCQ in this entry;
            }
        }

}}}
