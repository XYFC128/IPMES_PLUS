package org.ipmes;

import org.ipmes.pattern.Preprocess;
import org.json.JSONObject;

/**
 * EventEdge represent an event about to be fed into CEP tool
 */
public class EventEdge {
    public String timestamp;
    public String edgeId;
    public String edgeSignature;
    public String startId;
    public String startSignature;
    public String endId;
    public String endSignature;
    EventEdge(String ts, String eid, String eSig, String startId, String startSig, String endId, String endSig) {
        this.timestamp = ts;
        this.edgeId = eid;
        this.edgeSignature = eSig;
        this.startId = startId;
        this.startSignature = startSig;
        this.endId = endId;
        this.endSignature = endSig;
    }

    public EventEdge(JSONObject inpObj) {
        JSONObject edgeObj = inpObj.getJSONObject("r");
        this.timestamp = Preprocess.extractTimestamp(edgeObj);
        this.edgeId = edgeObj.getString("id");
        this.edgeSignature = Preprocess.extractEdgeSignature(edgeObj);
        this.startId = inpObj.getJSONObject("m").getString("id");
        this.startSignature = Preprocess.extractNodeSignature(inpObj.getJSONObject("m"));
        this.endId = inpObj.getJSONObject("n").getString("id");
        this.endSignature = Preprocess.extractNodeSignature(inpObj.getJSONObject("n"));
    }

    public EventEdge(String csvRow) {
        String[] fields = csvRow.split(",");
        this.timestamp      = fields[0];
        this.edgeId         = fields[1];
        this.edgeSignature  = fields[2];
        this.startId        = fields[3];
        this.startSignature = fields[4];
        this.endId          = fields[5];
        this.endSignature   = fields[6];
    }

    public Object[] toEventData() {
        return new Object[]{timestamp, edgeId, edgeSignature, startId, startSignature, endId, endSignature};
    }
}
