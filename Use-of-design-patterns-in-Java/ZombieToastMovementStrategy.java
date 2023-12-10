package dungeonmania.entities.enemies;

import dungeonmania.Game;
import dungeonmania.entities.collectables.potions.InvincibilityPotion;
import dungeonmania.map.GameMap;
import dungeonmania.util.Position;

public class ZombieToastMovementStrategy extends MovementStrategy {

    @Override
    public void move(Game game, Enemy enemy) {
        Position nextPos;
        GameMap map = game.getMap();
        ZombieToast zombieEnemy = (ZombieToast) enemy;
        if (map.getPlayer().getEffectivePotion() instanceof InvincibilityPotion) {
            // call the method here once as opposed to doing the same logic again in Mercenary
            nextPos = ifEffectedByInvinsibility(game, zombieEnemy);
        } else {
            // Again, call the method here once as opposed to doing the same logic again in Mercenary
            nextPos = moveRandom(game, zombieEnemy);
        }
        map.moveTo(zombieEnemy, nextPos);

    }


}
