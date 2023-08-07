package org.ipmes.decomposition;

/**
 * A tuple struct representing there is a relation between idOfResult and idOfEntry.
 * The ids are pattern edge ids.
 * TODO: Redesign this to an interface, inheritance by TemporalRelation and SpatialRelation
 */
public class TCQueryRelation {
    public Integer idOfResult;
    public Integer idOfEntry;

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        else if (!(o instanceof TCQueryRelation)) return false;
        TCQueryRelation other = (TCQueryRelation) o;
        return this.idOfResult.equals(other.idOfResult) && this.idOfEntry.equals(other.idOfEntry);
    }
}
