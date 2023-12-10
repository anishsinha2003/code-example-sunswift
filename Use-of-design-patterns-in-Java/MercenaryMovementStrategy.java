package dungeonmania.entities.enemies;

import dungeonmania.Game;
import dungeonmania.entities.Player;
import dungeonmania.entities.collectables.potions.InvincibilityPotion;
import dungeonmania.entities.collectables.potions.InvisibilityPotion;
import dungeonmania.map.GameMap;
import dungeonmania.util.Position;

public class MercenaryMovementStrategy extends MovementStrategy {
    @Override
    public void move(Game game, Enemy enemy) {
        Mercenary mercenaryEnemy = (Mercenary) enemy;

        Position nextPos;
        GameMap map = game.getMap();
        Player player = game.getPlayer();
        if (mercenaryEnemy.isAllied()) {
            nextPos = mercenaryEnemy.getIsAdjacentToPlayer() ? player.getPreviousDistinctPosition()
                    : map.dijkstraPathFind(mercenaryEnemy.getPosition(), player.getPosition(), mercenaryEnemy);
            if (!mercenaryEnemy.getIsAdjacentToPlayer() && Position.isAdjacent(player.getPosition(), nextPos))
                mercenaryEnemy.setIsAdjacentToPlayer(true);
        } else if (map.getPlayer().getEffectivePotion() instanceof InvisibilityPotion) {
            // call the method here once as opposed to doing the same logic again in ZombieToast
            nextPos = moveRandom(game, mercenaryEnemy);
        } else if (map.getPlayer().getEffectivePotion() instanceof InvincibilityPotion) {
            // Again, call the method here once as opposed to doing the same logic again in ZombieToast
            nextPos = ifEffectedByInvinsibility(game, mercenaryEnemy);
        } else {
            // Follow hostile
            nextPos = map.dijkstraPathFind(mercenaryEnemy.getPosition(), player.getPosition(), mercenaryEnemy);
        }

        map.moveTo(mercenaryEnemy, nextPos);

        // need to make a for loop that moves the head to newPos, then each following snake body to the old pos
    }

}
