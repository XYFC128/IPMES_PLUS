package org.ipmes.join;

import java.util.ArrayList;

import org.ipmes.decomposition.TCQuery;
import org.ipmes.decomposition.TCQueryRelation;
import org.ipmes.pattern.TemporalRelation;
import org.ipmes.pattern.PatternEdge;
import org.ipmes.pattern.PatternGraph;

public class NaiveGenRel {

    TemporalRelation temporalRelation;
    PatternGraph spatialRelation;
    ArrayList<TCQueryRelation>[] relations;

    public NaiveGenRel(TemporalRelation temporalRelation, PatternGraph spatialRelation, ArrayList<TCQuery> selected) {
        this.temporalRelation = temporalRelation;
        this.spatialRelation = spatialRelation;
        this.relations = genRelations(selected);
    }

    public ArrayList<TCQueryRelation>[] getRelation() {
        return this.relations;
    }

    /**
     * check whether two edges have any temporal or spatial relationship
     * 
     * @param edge1
     * @param edge2
     * @return true if have relation.
     */
    boolean hasRelations(PatternEdge edge1, PatternEdge edge2) {
        return this.temporalRelation.getParents(edge1.getId()).contains(edge2.getId())
                || this.temporalRelation.getChildren(edge1.getId()).contains(edge2.getId())
                || (!this.spatialRelation.getSharedNodes(edge1.getId(), edge2.getId()).isEmpty());
    }

    /**
     * Generate relations for all selected TC Query. This method will collect all
     * relations
     * related with all edges in a tc Query, including temporal relations and
     * spatial relations.
     * 
     * @param selected selected TC-Query
     * @return an array of relations, i-th element is the relations for i-th
     *         TC-Query
     */
    ArrayList<TCQueryRelation>[] genRelations(ArrayList<TCQuery> selected) {
        ArrayList<TCQueryRelation>[] relations = (ArrayList<TCQueryRelation>[]) new ArrayList[selected.size()];
        for (int i = 0; i < relations.length; i++) {
            relations[i] = new ArrayList<>();
            for (int j = 0; j < relations.length; j++) {
                if (i == j)
                    continue;
                for (PatternEdge edge1 : selected.get(i).getEdges()) {
                    for (PatternEdge edge2 : selected.get(j).getEdges()) {
                        if (hasRelations(edge1, edge2)) {
                            TCQueryRelation tempRelation = new TCQueryRelation();
                            tempRelation.idOfResult = edge1.getId();
                            tempRelation.idOfEntry = edge2.getId();
                            relations[i].add(tempRelation);
                        }
                    }
                }
            }
        }
        return relations;
    }
}
